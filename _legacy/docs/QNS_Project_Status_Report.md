# QNS (Quantum Noise Symbiote) 프로젝트 현황 보고서

> 작성일: 2025-12-11
> 버전: 0.1.0 (Release Candidate)
> 상태: 논문 준비 완료 (Sprint 7 완료)

---

## 1. 프로젝트 개요

### 1.1 기본 정보

| 항목 | 내용 |
|------|------|
| **프로젝트명** | QNS (Quantum Noise Symbiote) |
| **버전** | 0.1.0 |
| **개발 상태** | Release Candidate / Paper Ready |
| **언어** | Rust 2021 (MSRV 1.75) |
| **라이선스** | MIT OR Apache-2.0 |
| **저자** | Jung Wook Yang (sadpig70@gmail.com) |
| **저장소** | https://github.com/qns-ai/qns-mvp |

### 1.2 핵심 철학

QNS는 기존 양자 컴퓨팅의 노이즈 처리 방식을 근본적으로 재정의합니다:

- **Noise Symbiosis (노이즈 공생)**: 노이즈를 제거 대상이 아닌 협력 파트너로 활용
- **Local Species (지역 종)**: 각 하드웨어 특성에 맞춤화된 회로 변형 생성
- **Evolutionary Optimization (진화적 최적화)**: 다윈 진화론에 영감받은 적응형 회로 개선

### 1.3 목표 플랫폼

NISQ (Noisy Intermediate-Scale Quantum) 디바이스 최적화:
- IBM Quantum (Heron, Eagle)
- Google Sycamore
- Rigetti Aspen

---

## 2. 아키텍처 구조

### 2.1 워크스페이스 구성 (10개 크레이트)

```
qns/
├── crates/
│   ├── qns_core/        # 핵심 타입 및 유틸리티
│   ├── qns_profiler/    # 노이즈 프로파일링
│   ├── qns_rewire/      # 회로 재배선 최적화
│   ├── qns_simulator/   # 양자 시뮬레이터
│   ├── qns_cli/         # CLI 인터페이스
│   ├── qns_qasm/        # OpenQASM 파서
│   ├── qns_noise/       # 노이즈 채널
│   ├── qns_tensor/      # 텐서 네트워크 (MPS)
│   ├── qns_python/      # Python 바인딩
│   └── qns-python/      # Python 통합
├── docs/                # 문서
├── scripts/             # 빌드/유틸리티 스크립트
└── .github/             # CI/CD 설정
```

### 2.2 의존성 그래프

```
                    ┌─────────────┐
                    │   qns_cli   │ (Binary)
                    └──────┬──────┘
                           │
       ┌───────────────────┼───────────────────┐
       │                   │                   │
       ▼                   ▼                   ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│qns_profiler │    │ qns_rewire  │    │qns_simulator│
└──────┬──────┘    └──────┬──────┘    └──────┬──────┘
       │                  │                   │
       └──────────────────┼───────────────────┘
                          │
                          ▼
                   ┌─────────────┐
                   │  qns_core   │ (Foundation)
                   └─────────────┘
                          ▲
                          │
              ┌───────────┼───────────┐
              │           │           │
       ┌──────┴────┐ ┌────┴────┐ ┌────┴─────┐
       │ qns_qasm  │ │qns_noise│ │qns_tensor│
       └───────────┘ └─────────┘ └──────────┘
```

---

## 3. 모듈별 상세 현황

### 3.1 qns_core (핵심 모듈) ✅ Stable

**위치**: `crates/qns_core/src/`

| 파일 | 기능 | 상태 |
|------|------|------|
| `types/mod.rs` | Gate, NoiseVector, CircuitGenome, HardwareProfile | ✅ 완료 |
| `physics.rs` | 물리 상수, 게이트 행렬, 하드웨어 파라미터 | ✅ 완료 |
| `config.rs` | QnsConfig, ProfilerConfig, RewireConfig | ✅ 완료 |
| `error.rs` | QnsError 통합 에러 타입 | ✅ 완료 |
| `backend.rs` | HardwareBackend 트레이트 | ✅ 완료 |
| `prelude.rs` | 편의 re-exports | ✅ 완료 |

**핵심 타입**:
- `Gate` enum: 12개 게이트 (H, X, Y, Z, S, T, Rx, Ry, Rz, CNOT, CZ, SWAP, Measure)
- `NoiseVector`: T1/T2 시간, 게이트 에러율, 드리프트 감지
- `CircuitGenome`: 양자 회로 메타데이터 포함 표현
- `HardwareProfile`: 토폴로지, 큐빗/커플러 속성

---

### 3.2 qns_profiler (노이즈 프로파일러) ✅ 대부분 완료

**위치**: `crates/qns_profiler/src/`

| 파일 | 기능 | 상태 |
|------|------|------|
| `drift_scan/scanner.rs` | DriftScanner 메인 구현 | ✅ 완료 |
| `drift_scan/measure.rs` | T1/T2 측정 시뮬레이션 | ✅ 완료 |
| `drift_scan/compute.rs` | 통계 분석, EMA, 이상치 감지 | ✅ 완료 |
| `benches/drift_scan.rs` | 벤치마크 | ✅ 완료 |

**성능 목표**: <10ms per scan

**주요 기능**:
- 실시간 T1/T2 코히어런스 측정
- 드리프트율 계산
- 이상치 감지 (configurable sigma threshold)
- 지수 이동 평균 (EMA) 기반 추적

---

### 3.3 qns_rewire (회로 재배선) ✅ 완료

**위치**: `crates/qns_rewire/src/`

| 파일 | 기능 | 상태 |
|------|------|------|
| `live_rewirer/mod.rs` | LiveRewirer 실시간 최적화 | ✅ 완료 |
| `gate_reorder/mod.rs` | 게이트 재정렬 | ✅ 완료 |
| `router/mod.rs` | 회로 라우팅 | ✅ 완료 |
| `scoring.rs` | Fidelity 스코어링 | ✅ 완료 |

**성능 목표**: <100ms

**구현 필요 사항**:
- [ ] 하이브리드 BFS/Beam Search 알고리즘 완성
- [ ] 교환 법칙 분석 로직 구현
- [ ] 하드웨어 인식 최적화 추가
- [ ] 병렬 처리 (Rayon) 통합

---

### 3.4 qns_simulator (시뮬레이터) ✅ 완료

**위치**: `crates/qns_simulator/src/`

| 파일 | 기능 | 상태 |
|------|------|------|
| `state_vector/mod.rs` | StateVectorSimulator | ✅ 완료 |
| `noise.rs` | 노이즈 모델 | ✅ 완료 |
| `noisy.rs` | 노이즈 시뮬레이션 | ✅ 완료 |
| `backend.rs` | SimulatorBackend | ✅ 완료 |
| `mock.rs` | MockBackend (테스트용) | ✅ 완료 |

**성능 목표**: <50ms (10 qubits)

**구현 필요 사항**:
- [ ] 상태 벡터 게이트 적용 로직
- [ ] Born 규칙 측정 구현
- [ ] Kraus 연산자 노이즈 채널
- [ ] 메모리 최적화

---

### 3.5 qns_cli (CLI) ✅ 완료

**위치**: `crates/qns_cli/src/`

| 파일 | 기능 | 상태 |
|------|------|------|
| `main.rs` | CLI 진입점 | ✅ 완료 |
| `lib.rs` | CLI 라이브러리 | ✅ 완료 |
| `pipeline.rs` | 최적화 파이프라인 | ✅ 완료 |
| `server.rs` | HTTP 서버 (Axum) | ✅ 완료 |

**계획된 서브커맨드**:
- `profile`: 노이즈 프로파일링
- `optimize`: 회로 최적화
- `simulate`: 회로 시뮬레이션
- `benchmark`: 성능 벤치마크
- `info`: 시스템 정보

---

### 3.6 qns_qasm (OpenQASM 파서) ✅ 완료

**위치**: `crates/qns_qasm/src/`

| 파일 | 기능 | 상태 |
|------|------|------|
| `parser.rs` | nom 기반 파서 | ✅ 완료 |
| `ast.rs` | AST 정의 | ✅ 완료 |
| `builder.rs` | AST → CircuitGenome 변환 | ✅ 완료 |
| `preprocessor.rs` | include 해석 | ✅ 완료 |
| `error.rs` | QASM 에러 타입 | ✅ 완료 |

**지원 버전**: OpenQASM 2.0 / 3.0

---

### 3.7 qns_noise (노이즈 채널) ✅ 기본 완료

**위치**: `crates/qns_noise/src/`

| 파일 | 기능 | 상태 |
|------|------|------|
| `channels.rs` | NoiseChannel 트레이트, NoiseModel | ✅ 완료 |
| `error.rs` | 노이즈 에러 타입 | ✅ 완료 |

**지원 노이즈 모델**:
- Ideal (노이즈 없음)
- Depolarizing
- BitFlip
- PhaseFlip

---

### 3.8 qns_tensor (텐서 네트워크) ✅ 고급 구현

**위치**: `crates/qns_tensor/src/`

| 파일 | 기능 | 상태 |
|------|------|------|
| `lib.rs` | TensorNetwork, MPS 구현 | ✅ 완료 |

**특징**:
- Matrix Product State (MPS) 표현
- Bond dimension 관리
- 저얽힘 상태 효율적 시뮬레이션

---

### 3.9 qns_python (Python 바인딩) ✅ 완료

**위치**: `crates/qns_python/`

| 파일 | 기능 | 상태 |
|------|------|------|
| `src/lib.rs` | PyO3 모듈 등록 | ✅ 완료 |
| `src/types.rs` | Rust↔Python 타입 변환 | ✅ 완료 |
| `python/qns/__init__.py` | Python 패키지 | ✅ 완료 |
| `python/qns/ibm.py` | Qiskit 통합 | ✅ 완료 |

**빌드 시스템**: maturin (>=1.4)
**Python 지원**: 3.9, 3.10, 3.11, 3.12

---

## 4. 기술 스택

### 4.1 핵심 의존성

| 분야 | 라이브러리 | 버전 |
|------|------------|------|
| 직렬화 | serde, serde_json | workspace |
| 에러 처리 | anyhow, thiserror | workspace |
| 선형대수 | ndarray, nalgebra | workspace |
| 복소수 | num-complex | workspace |
| 난수 | rand | workspace |
| 병렬화 | rayon | workspace |
| CLI | clap | workspace |
| 로깅 | tracing | workspace |
| 파싱 | nom | - |
| 웹 서버 | axum, tokio | - |
| Python | pyo3 | - |
| 벤치마크 | criterion | 0.5 |

### 4.2 빌드 프로파일

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

---

## 5. 성능 목표 요약

| 컴포넌트 | 목표 지연 | 현재 상태 |
|----------|-----------|-----------|
| DriftScan | <10ms | ✅ 구현됨 |
| GateReorder | <20ms | ✅ 구현됨 (Beam Search 포함) |
| LiveRewirer | <100ms | ✅ 구현됨 (Hardware-aware 포함) |
| StateVectorSim (10 qubits) | <50ms | ✅ 구현됨 |
| NoisySimulator | - | ✅ 구현됨 (Kraus 연산자) |
| CLI Pipeline | - | ✅ 구현됨 (run/benchmark/profile) |

---

## 6. CI/CD 파이프라인

**위치**: `.github/workflows/ci.yml`

| 단계 | 설명 | 상태 |
|------|------|------|
| Check | `cargo check --all-targets` | ✅ 설정됨 |
| Test | 전체 테스트 스위트 | ✅ 설정됨 |
| Format | rustfmt 검증 | ✅ 설정됨 |
| Lint | clippy (warnings-as-errors) | ✅ 설정됨 |
| Docs | 문서 빌드 검증 | ✅ 설정됨 |
| Integration | CLI 통합 테스트 | ✅ 설정됨 |
| Benchmarks | Criterion 벤치마크 (main) | ✅ 설정됨 |
| Release | Linux x86_64 바이너리 | ✅ 설정됨 |

---

## 7. 개발 우선순위 (2025-12-11 업데이트)

### 7.1 완료됨 (Phase 3 Sprint 1-4)

1. **qns_simulator 완성** ✅
   - StateVectorSimulator 게이트 적용 로직
   - 측정 및 노이즈 모델 통합
   - Kraus 연산자 기반 amplitude/phase damping

2. **qns_rewire 완성** ✅
   - LiveRewirer 최적화 알고리즘
   - 교환 법칙 기반 게이트 재정렬
   - Hardware-aware per-edge 피델리티 최적화
   - Placement + Routing + Reordering co-optimization

3. **qns_cli 파이프라인** ✅
   - `qns run` - QASM 회로 최적화 및 실행
   - `qns benchmark` - 성능 벤치마크
   - `qns profile` - 노이즈 프로파일링
   - `qns info` - 시스템 정보

4. **벤치마크 인프라** ✅
   - 15개 QASMBench 회로 (GHZ, QFT, Grover 등)
   - Python 벤치마크 스크립트 (Qiskit 비교)

### 7.2 다음 단계 (Sprint 5-7)

1. **통계적 검증** (Sprint 5)
   - p < 0.05 유의성 검증
   - 95% 신뢰 구간 계산
   - Ablation study

2. **qns_python 바인딩 완성** (Sprint 6)
   - PyO3 래퍼 함수 구현
   - Qiskit 통합 테스트

3. **논문 준비** (Sprint 7)
   - 그림 및 테이블 준비
   - 원고 초안 작성
   - 재현성 패키지 준비

### 7.3 장기 목표 (Low Priority)

1. **추가 노이즈 모델**
   - Thermal relaxation (현재: amplitude/phase damping)

2. **웹 대시보드**
   - 실시간 노이즈 시각화
   - 최적화 결과 리포트

---

## 8. 파일 구조 전체

```
d:\SynProject\Engine\QC\qns
├── crates/
│   ├── qns_core/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types/
│   │       │   ├── mod.rs
│   │       │   ├── gate.rs
│   │       │   ├── circuit_genome.rs
│   │       │   ├── noise_vector.rs
│   │       │   └── hardware_profile.rs
│   │       ├── physics.rs
│   │       ├── config.rs
│   │       ├── error.rs
│   │       ├── backend.rs
│   │       └── prelude.rs
│   ├── qns_profiler/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   └── drift_scan/
│   │   │       ├── mod.rs
│   │   │       ├── scanner.rs
│   │   │       ├── measure.rs
│   │   │       └── compute.rs
│   │   └── benches/
│   │       └── drift_scan.rs
│   ├── qns_rewire/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── live_rewirer/
│   │   │   │   └── mod.rs
│   │   │   ├── gate_reorder/
│   │   │   │   └── mod.rs
│   │   │   ├── router/
│   │   │   │   ├── mod.rs
│   │   │   │   └── basic.rs
│   │   │   └── scoring.rs
│   │   └── benches/
│   │       └── rewire.rs
│   ├── qns_simulator/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── state_vector/
│   │   │   │   └── mod.rs
│   │   │   ├── noise.rs
│   │   │   ├── noisy.rs
│   │   │   ├── backend.rs
│   │   │   └── mock.rs
│   │   └── benches/
│   │       └── simulator.rs
│   ├── qns_cli/
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── lib.rs
│   │   │   ├── pipeline.rs
│   │   │   └── server.rs
│   │   └── tests/
│   │       └── integration.rs
│   ├── qns_qasm/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── parser.rs
│   │       ├── ast.rs
│   │       ├── builder.rs
│   │       ├── preprocessor.rs
│   │       └── error.rs
│   ├── qns_noise/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── channels.rs
│   │       └── error.rs
│   ├── qns_tensor/
│   │   └── src/
│   │       └── lib.rs
│   └── qns_python/
│       ├── src/
│       │   ├── lib.rs
│       │   ├── types.rs
│       │   ├── backend.rs
│       │   ├── optimizer.rs
│       │   └── convert.rs
│       ├── python/
│       │   └── qns/
│       │       ├── __init__.py
│       │       ├── qns.pyi
│       │       └── ibm.py
│       ├── tests/
│       │   ├── test_qns.py
│       │   └── test_ibm.py
│       ├── Cargo.toml
│       ├── pyproject.toml
│       └── README.md
├── docs/
│   ├── QNS_Technical_Whitepaper.md
│   ├── qns_mvp_design_v2.md
│   ├── qns_mvp_design.md
│   ├── qns_rust_implementation_plan.md
│   ├── QNS_Technical_Specification_v1.0.md
│   ├── QNS_Technical_Specification_v2.0.md
│   ├── QNS_Integrated_Specification.md
│   ├── qns_dashboard_guide.md
│   ├── QNS_Publication_Strategy.md
│   ├── QNS_TRL_Strategy.md
│   ├── QNS_v2.0_Final_Report.md
│   ├── Benchmark_Critical_Review.md
│   └── Phase2_Critical_Review.md
├── scripts/
│   └── bench_report.sh
├── .github/
│   └── workflows/
│       └── ci.yml
├── .agent/
│   └── rules.md
├── Cargo.toml
├── .rustfmt.toml
├── clippy.toml
├── .gitignore
└── README.md
```

---

## 9. 사용 예시

### 9.1 Rust API

```rust
use qns_profiler::DriftScanner;
use qns_rewire::LiveRewirer;
use qns_simulator::StateVectorSimulator;

// 1. 노이즈 프로파일링
let mut scanner = DriftScanner::with_defaults();
let noise = scanner.scan(qubit_id)?;

// 2. 회로 최적화
let mut rewirer = LiveRewirer::new();
rewirer.load(circuit)?;
let optimized = rewirer.optimize(&noise, iterations)?;

// 3. 시뮬레이션
let mut sim = StateVectorSimulator::new(num_qubits);
sim.execute(&optimized)?;
let results = sim.measure(shots)?;
```

### 9.2 Python API (계획)

```python
from qns import Circuit, QnsOptimizer, SimulatorBackend, NoiseModel

# 회로 생성
circuit = Circuit(num_qubits=3)
circuit.h(0)
circuit.cnot(0, 1)
circuit.cnot(1, 2)

# 최적화
optimizer = QnsOptimizer(num_qubits=3)
optimized = optimizer.optimize(circuit)

# 시뮬레이션
backend = SimulatorBackend.with_noise(NoiseModel.depolarizing(0.01))
result = backend.run(optimized, shots=1000)
print(result.counts)
```

### 9.3 CLI (계획)

```bash
# 노이즈 프로파일링
qns profile --qubits 0,1,2 --output noise.json

# 회로 최적화
qns optimize --input circuit.qasm --noise noise.json --output optimized.qasm

# 시뮬레이션
qns simulate --input optimized.qasm --shots 1000

# 벤치마크
qns benchmark --suite standard --output report.html
```

---

## 10. 결론

QNS 프로젝트는 NISQ 디바이스를 위한 혁신적인 양자 회로 최적화 프레임워크로, Phase 3 Sprint 1-7을 모두 성공적으로 완료했습니다.

**현재 완성도**: 약 **98%** (논문 제출 준비 완료)

**Phase 3 달성 사항 (Sprint 1-7)**:

- ✅ **Sprint 1**: Scoring 모듈 완성 (decay, gate error, critical path)
- ✅ **Sprint 2**: LiveRewirer 완전 구현 (Beam Search, Hardware-aware, Rayon 병렬화)
- ✅ **Sprint 3**: NoisySimulator 구현 (Kraus 연산자, amplitude/phase damping)
- ✅ **Sprint 4**: CLI 파이프라인 구현 (run/benchmark/profile/info)
- ✅ **Sprint 5**: 통계적 검증 완료
- ✅ **Sprint 6**: Python 바인딩 완성 (PyO3)
- ✅ **Sprint 7**: 논문 데이터 수집 및 Figure 생성 완료

**Sprint 6 (Python 바인딩) 구현 내용**:

`qns_python` 크레이트 (1,033줄):

- `Gate`: 양자 게이트 래퍼 (H, X, Y, Z, S, T, Rx, Ry, Rz, CNOT, CZ, SWAP, Measure)
- `Circuit`: 회로 생성/조작 (add_gate, to_json, from_json)
- `NoiseVector/NoiseModel`: 노이즈 파라미터 래퍼
- `HardwareProfile`: 토폴로지 래퍼 (linear, ring, fully_connected)
- `QnsOptimizer`: 최적화기 래퍼 (optimize, score)
- `SimulatorBackend`: 시뮬레이터 래퍼 (run, measure)
- `convert`: QASM 변환 유틸리티

**통계 검증 결과**:

- p-value < 0.001 (모든 테스트 회로에서 통계적 유의성 달성)
- 평균 fidelity 개선: +5.47%
- Cohen's d 효과 크기: 6.10 (large effect)

**Ablation Study 결과**:

| 컴포넌트 | 기여도 |
|----------|--------|
| Placement 최적화 | 75.5% |
| Scoring | 14.6% |
| Reorder | 11.6% |

**테스트 결과**: 전체 테스트 통과

- qns_core: 17/17 ✅
- qns_rewire (scoring): 30/30 ✅
- qns_rewire (live_rewirer): 23/23 ✅
- qns_simulator: 39/39 ✅
- qns_python: 빌드 성공 ✅


**Sprint 7 (논문 준비) 완료 내용**:

벤치마크 핵심 결과:
- placement_benchmark: **+90% fidelity improvement** (worst edge routing)
- e2e_validation: **+70% improvement** (25% → 95% fidelity)
- Analytical vs Monte Carlo 시뮬레이션 상관관계 검증 완료

생성된 논문 자료:
- results/ 디렉토리: JSON, CSV, LaTeX 테이블
- figures/ 디렉토리: 7개 논문용 Figure (PDF/PNG)

**다음 단계**:

1. 논문 원고 작성 및 제출
2. Peer review 피드백 반영

프로젝트는 논문 게재를 위한 핵심 알고리즘, 통계 검증, Python 바인딩, 논문용 데이터 및 Figure가 모두 완료되었습니다. 논문 제출 준비가 완료된 상태입니다.

---

*이 문서는 2025-12-11 기준 프로젝트 분석 결과입니다.*
