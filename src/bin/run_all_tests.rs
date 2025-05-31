use std::process::Command;
use std::io::{self, Write};
use std::env;
use dotenv::dotenv;

fn main() {
    // .env 파일에서 환경 변수 로드
    dotenv().ok();
    
    println!("===============================================");
    println!("🚀 솔라나 RPC 종합 성능 테스트를 시작합니다 🚀");
    println!("===============================================\n");
    
    // 환경 변수 확인
    let rpc_url = env::var("SOLANA_RPC_URL").unwrap_or_else(|_| {
        println!("⚠️ 환경변수 SOLANA_RPC_URL이 설정되지 않았습니다. 기본 RPC URL만 테스트합니다.");
        String::new()
    });
    
    if !rpc_url.is_empty() {
        println!("🔍 테스트할 커스텀 RPC URL: {}\n", rpc_url);
    }
    
    let has_private_key = env::var("SOLANA_PRIVATE_KEY").is_ok();
    if !has_private_key {
        println!("⚠️ 환경변수 SOLANA_PRIVATE_KEY가 설정되지 않았습니다. 트랜잭션 테스트는 건너뛰게 됩니다.\n");
    }
    
    // 테스트 순서 안내
    println!("다음 순서로 테스트가 진행됩니다:");
    println!("1. Ping 테스트 (네트워크 지연 시간)");
    println!("2. RPC API 벤치마크 (API 응답 시간)");
    if has_private_key {
        println!("3. 트랜잭션 속도 테스트 (실제 거래 처리 시간)\n");
    } else {
        println!("3. 트랜잭션 속도 테스트 (건너뜀 - 개인키 없음)\n");
    }
    
    // 계속 진행할지 확인
    print!("테스트를 시작하려면 Enter 키를 누르세요...");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    // 1. Ping 테스트 실행
    println!("\n\n===== 1️⃣ Ping 테스트 시작 =====\n");
    let status = Command::new("cargo")
        .args(["run", "--release", "--bin", "ping_test"])
        .status()
        .expect("Ping 테스트 실행 실패");
    
    if !status.success() {
        println!("⚠️ Ping 테스트가 비정상적으로 종료되었습니다.");
    }
    
    // 다음 테스트 전 잠시 대기
    println!("\n계속하려면 Enter 키를 누르세요...");
    io::stdout().flush().unwrap();
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    
    // 2. RPC 벤치마크 테스트 실행
    println!("\n\n===== 2️⃣ RPC API 벤치마크 테스트 시작 =====\n");
    let status = Command::new("cargo")
        .args(["run", "--release", "--bin", "rpc_benchmark"])
        .status()
        .expect("RPC 벤치마크 테스트 실행 실패");
    
    if !status.success() {
        println!("⚠️ RPC 벤치마크 테스트가 비정상적으로 종료되었습니다.");
    }
    
    // 개인키가 있는 경우에만 트랜잭션 테스트 실행
    if has_private_key {
        // 다음 테스트 전 잠시 대기
        println!("\n계속하려면 Enter 키를 누르세요...");
        io::stdout().flush().unwrap();
        input.clear();
        io::stdin().read_line(&mut input).unwrap();
        
        // 3. 트랜잭션 속도 테스트 실행
        println!("\n\n===== 3️⃣ 트랜잭션 속도 테스트 시작 =====\n");
        let status = Command::new("cargo")
            .args(["run", "--release", "--bin", "tx_speed_test"])
            .status()
            .expect("트랜잭션 속도 테스트 실행 실패");
        
        if !status.success() {
            println!("⚠️ 트랜잭션 속도 테스트가 비정상적으로 종료되었습니다.");
        }
    } else {
        println!("\n\n❌ 환경변수 SOLANA_PRIVATE_KEY가 설정되지 않아 트랜잭션 속도 테스트를 건너뜁니다.");
    }
    
    // 테스트 종료
    println!("\n===============================================");
    println!("🎉 모든 RPC 성능 테스트가 완료되었습니다 🎉");
    println!("===============================================");
} 