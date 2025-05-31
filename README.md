# 솔라나 토큰 전송 봇

솔라나 블록체인에서 SOL 토큰을 빠르게 전송할 수 있는 Rust 기반 CLI 도구입니다.

## 기능

- 커스텀 RPC URL 설정 가능
- 가스비(우선순위 수수료) 수동 설정 가능
- 실시간 토큰 전송 처리
- 개인키 기반 지갑 관리
- 환경 변수를 통한 RPC URL 및 개인키 관리
- RPC 서버 성능 벤치마크 도구

## 설치 방법

### 사전 요구사항

- Rust 및 Cargo가 설치되어 있어야 합니다. [Rust 설치 방법](https://www.rust-lang.org/tools/install)

### 빌드 및 실행

1. 저장소 클론

```powershell
git clone https://github.com/yourusername/solana_transfer_bot.git
cd solana_transfer_bot
```

2. 환경 변수 설정

환경 변수를 설정하는 두 가지 방법이 있습니다:

**A. PowerShell에서 직접 설정:**

```powershell
$env:SOLANA_RPC_URL = "https://api.mainnet-beta.solana.com"
$env:SOLANA_PRIVATE_KEY = "YOUR_PRIVATE_KEY_HERE"
```

**B. .env 파일 생성:**

프로젝트 루트 디렉토리에 `.env` 파일을 생성하고 다음 내용을 추가합니다:

```
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
SOLANA_PRIVATE_KEY=YOUR_PRIVATE_KEY_HERE
```

3. 빌드

```powershell
cargo build --release
```

4. 실행

```powershell
cargo run --release
```

## 사용 방법

1. 환경 변수를 설정하거나 프로그램 실행 시 RPC URL, 보내는 지갑의 개인키를 입력합니다.
2. 전송할 SOL 양과 가스비를 설정합니다.
3. 전송받을 주소를 입력하면 즉시 트랜잭션이 전송됩니다.
4. 프로그램을 종료하려면 'exit'를 입력하세요.

## RPC 성능 테스트 도구

프로젝트에는 RPC 서버의 성능을 테스트하기 위한 3가지 도구가 포함되어 있습니다:

### 1. RPC API 벤치마크

RPC API 응답 시간을 측정합니다:

```powershell
cargo run --release --bin rpc_benchmark
```

### 2. 네트워크 Ping 테스트

RPC 서버 호스트의 네트워크 지연시간을 측정합니다:

```powershell
cargo run --release --bin ping_test
```

### 3. 트랜잭션 속도 테스트

실제 트랜잭션 전송 및 확인 시간을 측정합니다 (환경 변수 필요):

```powershell
cargo run --release --bin tx_speed_test
```

> 참고: 트랜잭션 테스트에는 소량의 SOL이 사용됩니다 (자기 자신에게 전송).

## 보안 참고사항

- 개인키는 안전하게 관리하세요. 이 프로그램은 테스트 및 개인 사용 목적으로만 사용하는 것이 좋습니다.
- `.env` 파일을 사용할 경우 `.gitignore`에 추가하여 실수로 공개 저장소에 업로드되지 않도록 주의하세요.
- 실제 운영 환경에서는 개인키를 환경 변수나 보안 저장소에서 관리하는 것이 안전합니다.

## 라이센스

MIT
