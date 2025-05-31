use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
    compute_budget::ComputeBudgetInstruction,
};
use std::env;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use dotenv::dotenv;

// 테스트할 RPC 엔드포인트 목록
const OFFICIAL_RPC: &str = "https://api.mainnet-beta.solana.com";

// 테스트 트랜잭션 수
const TEST_TX_COUNT: usize = 3;

// 테스트에 사용할 SOL 금액 (0.000001 SOL = 1000 lamports)
const TEST_LAMPORTS: u64 = 1000;

fn main() {
    // .env 파일에서 환경 변수 로드
    dotenv().ok();
    
    println!("솔라나 RPC 서버 트랜잭션 속도 테스트를 시작합니다...");
    println!("각 RPC 서버의 트랜잭션 처리 시간을 측정합니다.\n");
    
    // 환경 변수에서 RPC URL 불러오기
    let custom_rpc_url = env::var("SOLANA_RPC_URL").unwrap_or_else(|_| {
        println!("환경변수 SOLANA_RPC_URL이 설정되지 않았습니다. 기본 RPC URL만 테스트합니다.");
        String::new()
    });
    
    // RPC 엔드포인트 목록 생성
    let mut rpc_endpoints = vec![OFFICIAL_RPC];
    if !custom_rpc_url.is_empty() {
        rpc_endpoints.push(&custom_rpc_url);
    }
    
    // 환경 변수에서 개인키 불러오기
    let private_key = env::var("SOLANA_PRIVATE_KEY").unwrap_or_else(|_| {
        panic!("환경변수 SOLANA_PRIVATE_KEY가 설정되지 않았습니다.");
    });
    
    let sender_keypair = create_keypair_from_base58(&private_key);
    println!("테스트 지갑 주소: {}", sender_keypair.pubkey());
    
    // 결과를 저장할 맵
    let mut results: HashMap<&str, Vec<Duration>> = HashMap::new();
    
    // 각 RPC 엔드포인트에 대해 테스트 실행
    for &endpoint in &rpc_endpoints {
        println!("\n테스트 중: {}", endpoint);
        
        // RPC 클라이언트 생성
        let client = RpcClient::new_with_timeout(endpoint.to_string(), Duration::from_secs(30));
        
        // 지갑 잔액 확인
        match client.get_balance(&sender_keypair.pubkey()) {
            Ok(balance) => {
                println!("  현재 잔액: {} SOL", balance as f64 / 1_000_000_000.0);
                if balance < TEST_LAMPORTS * TEST_TX_COUNT as u64 + 10000 {
                    println!("  경고: 잔액이 부족합니다. 테스트를 위해 최소 0.00001 SOL이 필요합니다.");
                    continue;
                }
            },
            Err(e) => {
                println!("  잔액 확인 실패: {:?}", e);
                continue;
            }
        }
        
        // 결과 벡터 초기화
        results.insert(endpoint, Vec::new());
        
        // 테스트 수신자 생성 (자기 자신에게 전송)
        let recipient = sender_keypair.pubkey();
        
        // 여러 번 트랜잭션 전송 테스트
        for i in 1..=TEST_TX_COUNT {
            println!("  트랜잭션 테스트 #{}", i);
            
            // 트랜잭션 전송 및 시간 측정
            match send_sol_and_measure_time(&client, &sender_keypair, &recipient, TEST_LAMPORTS) {
                Ok(elapsed) => {
                    println!("    처리 시간: {:?}", elapsed);
                    results.get_mut(endpoint).unwrap().push(elapsed);
                },
                Err(e) => {
                    println!("    오류: {:?}", e);
                }
            }
        }
    }
    
    // 종합 결과 출력
    println!("\n===== 트랜잭션 처리 속도 결과 =====");
    println!("| RPC 엔드포인트 | 평균 처리시간 | 최소 처리시간 | 최대 처리시간 |");
    println!("|----------------|--------------|--------------|--------------|");
    
    let mut avg_times: Vec<(String, Duration)> = Vec::new();
    
    for (&endpoint, durations) in &results {
        if durations.is_empty() {
            println!("| {:<14} | 테스트 실패   | -            | -            |", endpoint);
            continue;
        }
        
        // 통계 계산
        let sum: Duration = durations.iter().sum();
        let avg = sum / durations.len() as u32;
        let min = durations.iter().min().unwrap();
        let max = durations.iter().max().unwrap();
        
        println!("| {:<14} | {:?} | {:?} | {:?} |", 
            endpoint, avg, min, max);
            
        avg_times.push((endpoint.to_string(), avg));
    }
    
    // 가장 빠른 RPC 서버 확인
    if !avg_times.is_empty() {
        avg_times.sort_by_key(|(_, duration)| *duration);
        println!("\n🏆 가장 빠른 트랜잭션 처리 RPC 서버: {} (평균: {:?})", avg_times[0].0, avg_times[0].1);
    }
}

// Base58 인코딩된 개인키로부터 Keypair 생성
fn create_keypair_from_base58(private_key: &str) -> Keypair {
    let bytes = bs58::decode(private_key)
        .into_vec()
        .expect("유효하지 않은 개인키");
    
    Keypair::from_bytes(&bytes).expect("키페어 생성 실패")
}

// SOL 전송 및 시간 측정 함수
fn send_sol_and_measure_time(
    rpc_client: &RpcClient,
    sender: &Keypair,
    recipient: &Pubkey,
    lamports: u64,
) -> Result<Duration, Box<dyn std::error::Error>> {
    // 시작 시간 기록
    let start_time = Instant::now();
    
    // 최근 블록해시 가져오기
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    
    // 명령어 벡터 생성
    let mut instructions = vec![];
    
    // 우선순위 수수료 설정 (빠른 확인을 위해)
    let priority_fee_instruction = ComputeBudgetInstruction::set_compute_unit_price(5);
    instructions.push(priority_fee_instruction);
    
    // 전송 명령 추가
    let transfer_instruction = system_instruction::transfer(&sender.pubkey(), recipient, lamports);
    instructions.push(transfer_instruction);
    
    // 트랜잭션 생성 및 서명
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&sender.pubkey()));
    transaction.sign(&[sender], recent_blockhash);
    
    // 트랜잭션 전송 및 확인
    println!("    트랜잭션 전송 중...");
    let signature = rpc_client.send_and_confirm_transaction_with_spinner_and_commitment(
        &transaction,
        CommitmentConfig::confirmed(),
    )?;
    
    // 종료 시간 기록 및 소요 시간 계산
    let elapsed = start_time.elapsed();
    println!("    트랜잭션 확인: {}", signature);
    
    Ok(elapsed)
} 