use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::env;
use dotenv::dotenv;

// 공식 RPC 엔드포인트
const OFFICIAL_RPC: &str = "https://api.mainnet-beta.solana.com";

fn main() {
    // .env 파일에서 환경 변수 로드
    dotenv().ok();
    
    println!("솔라나 RPC 서버 성능 벤치마크 테스트를 시작합니다...");
    println!("각 RPC 서버에 대해 다양한 작업의 응답 시간을 측정합니다.\n");

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
    
    // 결과를 저장할 맵
    let mut results: HashMap<&str, Vec<Duration>> = HashMap::new();

    // 각 RPC 엔드포인트에 대해 테스트 실행
    for &endpoint in &rpc_endpoints {
        println!("테스트 중: {}", endpoint);
        
        // RPC 클라이언트 생성
        let client = match RpcClient::new_with_timeout(endpoint.to_string(), Duration::from_secs(30)) {
            client => client,
        };

        // 결과 벡터 초기화
        results.insert(endpoint, Vec::new());
        
        // 테스트 1: 연결 테스트 (getVersion)
        print!("  - 버전 확인: ");
        let start = Instant::now();
        match client.get_version() {
            Ok(_) => {
                let elapsed = start.elapsed();
                println!("{:?}", elapsed);
                results.get_mut(endpoint).unwrap().push(elapsed);
            },
            Err(e) => {
                println!("오류: {:?}", e);
                results.get_mut(endpoint).unwrap().push(Duration::from_secs(999));
            }
        }
        
        // 테스트 2: 블록 해시 가져오기
        print!("  - 최근 블록해시 가져오기: ");
        let start = Instant::now();
        match client.get_latest_blockhash() {
            Ok(_) => {
                let elapsed = start.elapsed();
                println!("{:?}", elapsed);
                results.get_mut(endpoint).unwrap().push(elapsed);
            },
            Err(e) => {
                println!("오류: {:?}", e);
                results.get_mut(endpoint).unwrap().push(Duration::from_secs(999));
            }
        }
        
        // 테스트 3: 슬롯 정보 가져오기
        print!("  - 현재 슬롯 가져오기: ");
        let start = Instant::now();
        match client.get_slot_with_commitment(CommitmentConfig::confirmed()) {
            Ok(_) => {
                let elapsed = start.elapsed();
                println!("{:?}", elapsed);
                results.get_mut(endpoint).unwrap().push(elapsed);
            },
            Err(e) => {
                println!("오류: {:?}", e);
                results.get_mut(endpoint).unwrap().push(Duration::from_secs(999));
            }
        }
        
        // 테스트 4: 지갑 잔액 확인 (큰 지갑 사용)
        print!("  - 대형 지갑 잔액 확인: ");
        let start = Instant::now();
        match client.get_balance(&"4Rf9mGD7FeYknun5JczX5nGLTfQuS1GRjwA3iseBQxP4".parse().unwrap()) {
            Ok(_) => {
                let elapsed = start.elapsed();
                println!("{:?}", elapsed);
                results.get_mut(endpoint).unwrap().push(elapsed);
            },
            Err(e) => {
                println!("오류: {:?}", e);
                results.get_mut(endpoint).unwrap().push(Duration::from_secs(999));
            }
        }
        
        println!();
    }
    
    // 종합 결과 출력
    println!("\n===== 종합 결과 =====");
    println!("| RPC 엔드포인트 | 평균 응답시간 | 최소 응답시간 | 최대 응답시간 |");
    println!("|----------------|--------------|--------------|--------------|");
    
    // 평균 시간을 저장할 벡터 (문자열로 저장하여 참조 문제 해결)
    let mut avg_times: Vec<(String, Duration)> = Vec::new();
    
    for (&endpoint, durations) in &results {
        // 유효한 응답만 필터링 (오류가 발생한 경우 제외)
        let valid_durations: Vec<&Duration> = durations.iter()
            .filter(|&&d| d != Duration::from_secs(999))
            .collect();
            
        if valid_durations.is_empty() {
            println!("| {:<14} | 연결 오류     | -            | -            |", endpoint);
            continue;
        }
        
        // 통계 계산
        let sum: Duration = valid_durations.iter().fold(Duration::from_secs(0), |acc, &x| acc + *x);
        let avg = sum / valid_durations.len() as u32;
        let min = valid_durations.iter().min().unwrap();
        let max = valid_durations.iter().max().unwrap();
        
        println!("| {:<14} | {:?} | {:?} | {:?} |", 
            endpoint, avg, min, max);
            
        // 문자열로 변환하여 저장
        avg_times.push((endpoint.to_string(), avg));
    }
    
    // 가장 빠른 RPC 서버 확인
    if !avg_times.is_empty() {
        avg_times.sort_by_key(|(_, duration)| *duration);
        println!("\n🏆 가장 빠른 RPC 서버: {} (평균: {:?})", avg_times[0].0, avg_times[0].1);
    }
} 