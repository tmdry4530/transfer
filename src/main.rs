use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
    compute_budget::ComputeBudgetInstruction,
};
use std::io::{self, BufRead, Write};
use std::str::FromStr;
use std::time::{Duration, Instant};
use std::env;
use dotenv::dotenv;

fn main() {
    // .env 파일에서 환경 변수 로드
    dotenv().ok();
    
    println!("솔라나 토큰 전송 봇이 시작되었습니다.");
    
    // 환경 변수에서 RPC URL 불러오기
    let rpc_url = env::var("SOLANA_RPC_URL").unwrap_or_else(|_| {
        let input = get_input("환경변수 SOLANA_RPC_URL이 설정되지 않았습니다. RPC URL을 입력하세요: ");
        input.trim().to_string()
    });
    
    println!("사용 중인 RPC URL: {}", rpc_url);
    
    // 환경 변수에서 개인키 불러오기
    let private_key = env::var("SOLANA_PRIVATE_KEY").unwrap_or_else(|_| {
        let input = get_input("환경변수 SOLANA_PRIVATE_KEY가 설정되지 않았습니다. 개인키를 입력하세요: ");
        input.trim().to_string()
    });
    
    let sender_keypair = create_keypair_from_base58(&private_key);
    println!("지갑 주소: {}", sender_keypair.pubkey());
    
    // 전송할 SOL 양 입력 받기
    let amount_input = get_input("전송할 SOL 양을 입력하세요: ");
    let amount: f64 = amount_input.trim().parse().expect("유효한 숫자를 입력하세요");
    
    // 우선순위 수수료 설정 (lamports per compute unit)
    let default_fee = 1;
    let fee_input = get_input(format!("가스비를 lamports/compute unit 단위로 설정하세요 (기본값: {}): ", default_fee).as_str());
    let fee: u64 = if fee_input.trim().is_empty() {
        default_fee
    } else {
        fee_input.trim().parse().expect("유효한 숫자를 입력하세요")
    };
    
    // RPC 클라이언트 초기화
    println!("RPC 연결 중: {}", rpc_url);
    let rpc_client = RpcClient::new_with_timeout(rpc_url, Duration::from_secs(30));
    
    // 전송 프로세스 시작
    loop {
        let recipient_input = get_input("전송받을 주소를 입력하세요 (종료하려면 'exit' 입력): ");
        
        if recipient_input.trim().to_lowercase() == "exit" {
            println!("프로그램을 종료합니다.");
            break;
        }
        
        // 수신자 주소 파싱
        let recipient_pubkey = match Pubkey::from_str(recipient_input.trim()) {
            Ok(pubkey) => pubkey,
            Err(_) => {
                println!("오류: 유효하지 않은 솔라나 주소입니다.");
                continue;
            }
        };
        
        // SOL을 lamports로 변환 (1 SOL = 10^9 lamports)
        let lamports = (amount * 1_000_000_000.0) as u64;
        
        // 전송 실행
        match send_sol(&rpc_client, &sender_keypair, &recipient_pubkey, lamports, fee) {
            Ok(signature) => {
                println!("전송 성공! 트랜잭션 서명: {}", signature);
                println!("트랜잭션 확인: https://explorer.solana.com/tx/{}?cluster=mainnet", signature);
            }
            Err(err) => {
                println!("전송 실패: {}", err);
            }
        }
    }
}

// 사용자 입력을 받는 함수
fn get_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().expect("출력 실패");
    
    let mut input = String::new();
    io::stdin().lock().read_line(&mut input).expect("입력 읽기 실패");
    input
}

// Base58 인코딩된 개인키로부터 Keypair 생성
fn create_keypair_from_base58(private_key: &str) -> Keypair {
    let bytes = bs58::decode(private_key)
        .into_vec()
        .expect("유효하지 않은 개인키");
    
    Keypair::from_bytes(&bytes).expect("키페어 생성 실패")
}

// SOL 전송 함수
fn send_sol(
    rpc_client: &RpcClient,
    sender: &Keypair,
    recipient: &Pubkey,
    lamports: u64,
    fee: u64,
) -> Result<String, Box<dyn std::error::Error>> {
    // 최근 블록해시 가져오기
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    
    // 명령어 벡터 생성
    let mut instructions = vec![];
    
    // 우선순위 수수료 설정 (최신 SDK 방식)
    if fee > 0 {
        // 수수료 우선순위 설정 명령어 추가
        let fee_instruction = ComputeBudgetInstruction::set_compute_unit_price(fee);
        instructions.push(fee_instruction);
    }
    
    // 전송 명령 추가
    let transfer_instruction = system_instruction::transfer(&sender.pubkey(), recipient, lamports);
    instructions.push(transfer_instruction);
    
    // 트랜잭션 생성 및 서명
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&sender.pubkey()));
    transaction.sign(&[sender], recent_blockhash);
    
    // 트랜잭션 전송
    let start_time = Instant::now();
    let signature = rpc_client.send_and_confirm_transaction_with_spinner_and_commitment(
        &transaction,
        CommitmentConfig::confirmed(),
    )?;
    let elapsed = start_time.elapsed();
    println!("트랜잭션 처리 시간: {:?}", elapsed);
    
    Ok(signature.to_string())
} 