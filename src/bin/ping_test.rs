use std::process::Command;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::thread;
use std::env;
use url::Url;
use dotenv::dotenv;

// 공식 RPC 엔드포인트
const OFFICIAL_RPC: &str = "https://api.mainnet-beta.solana.com";

fn main() {
    // .env 파일에서 환경 변수 로드
    dotenv().ok();
    
    println!("솔라나 RPC 서버 Ping 테스트를 시작합니다...");
    println!("각 RPC 서버의 네트워크 지연 시간을 측정합니다.\n");

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
        
        // URL에서 호스트 추출
        let host = match extract_host(endpoint) {
            Some(host) => host,
            None => {
                println!("  유효하지 않은 URL: {}", endpoint);
                continue;
            }
        };
        
        println!("  호스트: {}", host);
        
        // 결과 벡터 초기화
        results.insert(endpoint, Vec::new());
        
        // Ping 테스트 실행 (5회)
        for i in 1..=5 {
            print!("  Ping #{}: ", i);
            
            let start = Instant::now();
            
            #[cfg(target_os = "windows")]
            let ping_result = Command::new("ping")
                .args(&["-n", "1", &host])
                .output();
                
            #[cfg(not(target_os = "windows"))]
            let ping_result = Command::new("ping")
                .args(&["-c", "1", &host])
                .output();
            
            match ping_result {
                Ok(output) => {
                    let _elapsed = start.elapsed();
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    
                    // Ping 시간 추출
                    if let Some(time) = extract_ping_time(&output_str) {
                        println!("{:?}", time);
                        results.get_mut(endpoint).unwrap().push(time);
                    } else {
                        println!("응답 없음");
                        results.get_mut(endpoint).unwrap().push(Duration::from_secs(999));
                    }
                },
                Err(e) => {
                    println!("오류: {:?}", e);
                    results.get_mut(endpoint).unwrap().push(Duration::from_secs(999));
                }
            }
            
            // 다음 ping 전 약간의 지연
            thread::sleep(Duration::from_millis(200));
        }
        
        println!();
    }
    
    // 종합 결과 출력
    println!("\n===== Ping 테스트 결과 =====");
    println!("| RPC 엔드포인트 | 평균 지연시간 | 최소 지연시간 | 최대 지연시간 |");
    println!("|----------------|--------------|--------------|--------------|");
    
    let mut avg_times: Vec<(String, Duration)> = Vec::new();
    
    for (&endpoint, durations) in &results {
        // 유효한 응답만 필터링 (오류가 발생한 경우 제외)
        let valid_durations: Vec<&Duration> = durations.iter()
            .filter(|&&d| d != Duration::from_secs(999))
            .collect();
            
        if valid_durations.is_empty() {
            println!("| {:<14} | 응답 없음     | -            | -            |", endpoint);
            continue;
        }
        
        // 통계 계산
        let sum: Duration = valid_durations.iter().fold(Duration::from_secs(0), |acc, &x| acc + *x);
        let avg = sum / valid_durations.len() as u32;
        let min = valid_durations.iter().min().unwrap();
        let max = valid_durations.iter().max().unwrap();
        
        println!("| {:<14} | {:?} | {:?} | {:?} |", 
            endpoint, avg, min, max);
            
        avg_times.push((endpoint.to_string(), avg));
    }
    
    // 가장 빠른 RPC 서버 확인
    if !avg_times.is_empty() {
        avg_times.sort_by_key(|(_, duration)| *duration);
        println!("\n🏆 가장 낮은 지연시간 RPC 서버: {} (평균: {:?})", avg_times[0].0, avg_times[0].1);
    }
}

// URL에서 호스트 추출
fn extract_host(url_str: &str) -> Option<String> {
    match Url::parse(url_str) {
        Ok(url) => url.host_str().map(|s| s.to_string()),
        Err(_) => None,
    }
}

// Ping 결과에서 시간 추출
fn extract_ping_time(ping_output: &str) -> Option<Duration> {
    // Windows 형식의 ping 출력에서 시간 추출
    #[cfg(target_os = "windows")]
    {
        let time_str = ping_output.lines()
            .find(|line| line.contains("시간=") || line.contains("time="))
            .and_then(|line| {
                line.split("=").nth(3).or_else(|| line.split("=").nth(2))
            })
            .and_then(|s| s.trim_end_matches("ms").trim().parse::<u64>().ok());
            
        time_str.map(|ms| Duration::from_millis(ms))
    }
    
    // Unix 형식의 ping 출력에서 시간 추출
    #[cfg(not(target_os = "windows"))]
    {
        let time_str = ping_output.lines()
            .find(|line| line.contains("time="))
            .and_then(|line| {
                line.split("time=").nth(1)
            })
            .and_then(|s| s.trim_end_matches(" ms").trim().parse::<f64>().ok());
            
        time_str.map(|ms| Duration::from_millis(ms as u64))
    }
} 