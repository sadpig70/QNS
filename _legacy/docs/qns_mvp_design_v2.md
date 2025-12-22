# QNS MVP - Gantree 설계 v2.0 (수정본)

**버전**: 2.0  
**작성일**: 2025-11-26  
**기반**: v1.1 검증 완료본 + 구조 개선  
**변경사항**: 데이터 흐름 수정, QNS_Core 추가, Gate 확장, 의존성 보완

---

## 🔄 v1.1 → v2.0 주요 변경사항

| 항목 | v1.1 | v2.0 |
|------|------|------|
| Gantree 루트 | QNS_Profiler, QNS_Rewire | **QNS_Core 추가** |
| 데이터 흐름 | 순환 의존성 | **단방향 파이프라인** |
| Gate enum | 3개 | **12개** |
| 프로젝트 구조 | 혼재 | **워크스페이스 단일화** |
| rayon 의존성 | 누락 | **추가** |
| Result 타입 | 미정의 | **정의 완료** |

---

## 📐 Level 0: 시스템 루트 (수정)

```gantree
QNS_MVP_System // 양자 노이즈 공생 MVP 시스템 (설계중)
    QNS_Core // 핵심 타입 시스템 (설계중)
    QNS_Profiler // 노이즈 프로파일러 모듈 (진행중)
    QNS_Rewire // 회로 재배선 모듈 (진행중)
    QNS_Simulator // 양자 시뮬레이터 모듈 (설계중)
```

---

## 🧱 Level 1-4: QNS_Core (신규)

```gantree
QNS_Core // 핵심 타입 시스템 (설계중)
    CoreTypes // 핵심 데이터 타입 (설계중)
        NoiseVector // 노이즈 벡터 구조체 (설계중)
            DefineFields // 필드 정의 (설계중)
            ImplTraits // Default, Debug, Clone, Serialize (설계중)
            ImplMethods // new(), is_anomaly() (설계중)
        CircuitGenome // 회로 게놈 구조체 (설계중)
            DefineFields // num_qubits, gates, paths (설계중)
            ImplTraits // Debug, Clone, Serialize (설계중)
            ImplMethods // new(), add_gate(), depth() (설계중)
        Gate // 양자 게이트 열거형 (설계중)
            SingleQubitGates // H, X, Y, Z, S, T, Rx, Ry, Rz (설계중)
            TwoQubitGates // CNOT, CZ, SWAP (설계중)
            MeasureGate // Measure (설계중)
        HardwareProfile // 하드웨어 프로파일 (설계중)
            DefineFields // vendor, model, topology (설계중)
    ErrorTypes // 에러 타입 시스템 (설계중)
        QnsError // 통합 에러 열거형 (설계중)
            ProfilerError // 프로파일러 에러 (설계중)
            RewireError // 재배선 에러 (설계중)
            SimulatorError // 시뮬레이터 에러 (설계중)
            IoError // I/O 에러 (설계중)
        ResultType // Result<T> 타입 alias (설계중)
    Config // 설정 관리 (설계중)
        QnsConfig // 전역 설정 구조체 (설계중)
        LoadConfig // 설정 로드 (설계중)
```

**노드 수**: 24개  
**최대 깊이**: 4레벨  
**상태**: 🆕 신규

---

## 📊 Level 1-4: QNS_Profiler (유지)

```gantree
QNS_Profiler // 노이즈 프로파일러 모듈 (진행중)
    DriftScan // T1/T2 드리프트 스캐너 (완료)
        DriftScanner // 스캐너 구조체 (완료)
            NewWithConfig // 설정 기반 생성자 (완료)
        MeasureT1T2 // T1/T2 시간 측정 (완료)
            SimulateT1 // T1 시간 시뮬레이션 (완료)
            SimulateT2 // T2 시간 시뮬레이션 (완료)
            CollectSamples // 샘플 수집 - 1000 shots (완료)
        ComputeDriftVector // 드리프트 벡터 계산 (완료)
            CalculateStats // 평균/표준편차 통합 계산 (완료)
            ComputeDriftRate // 드리프트 속도 계산 (완료)
        DetectAnomaly // 이상 감지 (완료)
            CheckThreshold // 임계값 초과 확인 (완료)
            TriggerAlert // 알림 트리거 (완료)
    BurstDetector // 노이즈 버스트 감지기 (설계중)
        BurstMonitor // 버스트 모니터 구조체 (설계중)
        DetectSpike // 스파이크 감지 (설계중)
        GenerateEvent // 이벤트 생성 (설계중)
```

**노드 수**: 17개 (원자화 노드 통합으로 감소)  
**최대 깊이**: 4레벨  
**상태**: ✅ DriftScan 완료

---

## 🔄 Level 1-4: QNS_Rewire (수정)

```gantree
QNS_Rewire // 회로 재배선 모듈 (진행중)
    GateReorder // 게이트 재배열 연산자 (완료)
        ReorderOperator // 재배열 연산자 구조체 (완료)
            NewOperator // 연산자 생성 (완료)
        FindCommutingGates // 교환 가능한 게이트 찾기 (완료)
            CheckCommutativity // 교환법칙 확인 (완료)
            GroupCommutingPairs // 교환 가능 쌍 그룹화 (완료)
        GenerateReorderings // 재배열 생성 (완료)
            PermuteGateOrder // 게이트 순서 치환 (완료)
            CreateVariant // 변종 회로 생성 (완료)
    LiveRewirer // 실시간 재배선 엔진 (완료)
        RewireEngine // 재배선 엔진 구조체 (완료)
            NewEngine // 엔진 생성 (완료)
            LoadCircuit // 회로 로드 (완료)
        AnalyzeCircuit // 회로 분석 (완료)
            ExtractGates // 게이트 추출 (완료)
            IdentifyDependencies // 의존성 식별 (완료)
            BuildDAG // DAG 구축 (완료)
        GenerateVariants // 변종 생성 (완료)
            CallGateReorder // GateReorder 호출 (완료)
            ApplyNoiseAware // 노이즈 인지 변형 (완료)
        SelectBest // 최적 회로 선택 (완료)
            EvaluateFitness // 적합도 평가 (완료)
            RankVariants // 변종 순위화 (완료)
    CircuitValidator // 회로 검증기 (설계중)
        ValidateStructure // 구조 검증 (설계중)
        ValidateSemantics // 의미론 검증 (설계중)
```

**노드 수**: 24개  
**최대 깊이**: 4레벨  
**상태**: ✅ GateReorder, LiveRewirer 완료

---

## 🎮 Level 1-4: QNS_Simulator (신규 상세화)

```gantree
QNS_Simulator // 양자 시뮬레이터 모듈 (설계중)
    SimulatorTrait // 시뮬레이터 트레잇 (설계중)
        DefineInterface // 인터페이스 정의 (설계중)
    StateVectorSim // 상태벡터 시뮬레이터 (설계중)
        StateVector // 상태 벡터 구조체 (설계중)
            NewStateVector // 초기화 (설계중)
            ApplyGate // 게이트 적용 (설계중)
        ExecuteCircuit // 회로 실행 (설계중)
            ParseGates // 게이트 파싱 (설계중)
            SequentialApply // 순차 적용 (설계중)
        Measure // 측정 (설계중)
            CollapseState // 상태 붕괴 (설계중)
            SampleOutcome // 결과 샘플링 (설계중)
        CalcFidelity // 충실도 계산 (설계중)
            InnerProduct // 내적 계산 (설계중)
    NoiseModel // 노이즈 모델 (설계중)
        DepolarizingNoise // 탈분극 노이즈 (설계중)
        AmplitudeDamping // 진폭 감쇠 (설계중)
```

**노드 수**: 16개  
**최대 깊이**: 4레벨  
**성능 목표**: <50ms (10큐비트 기준)

---

## 🔀 데이터 흐름 정의 (수정)

### 수정 전 (v1.1) - 순환 의존성 문제
```
DriftScan ──▶ NoiseVector
                  │
LiveRewirer ◀────┘
     │
     ▼ (inputs: reordered_circuits) ← ❌ GateReorder 출력 참조
GateReorder
```

### 수정 후 (v2.0) - 단방향 파이프라인
```
┌─────────────────────────────────────────────────────────┐
│                    QNS Pipeline                          │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌──────────┐    NoiseVector    ┌─────────────┐         │
│  │DriftScan │ ─────────────────▶│             │         │
│  └──────────┘                   │             │         │
│                                 │ LiveRewirer │         │
│  ┌──────────┐   Vec<Circuit>    │  (통합)     │         │
│  │  Circuit │ ─────────────────▶│             │         │
│  └──────────┘                   │             │         │
│                                 └──────┬──────┘         │
│                                        │                 │
│                    ┌───────────────────┼────────────┐   │
│                    │ LiveRewirer 내부  │            │   │
│                    │                   ▼            │   │
│                    │  ┌─────────────────────────┐  │   │
│                    │  │ 1. GateReorder 호출     │  │   │
│                    │  │    → Vec<CircuitGenome> │  │   │
│                    │  └───────────┬─────────────┘  │   │
│                    │              │                 │   │
│                    │              ▼                 │   │
│                    │  ┌─────────────────────────┐  │   │
│                    │  │ 2. NoiseAware 변형      │  │   │
│                    │  │    → 노이즈 기반 조정   │  │   │
│                    │  └───────────┬─────────────┘  │   │
│                    │              │                 │   │
│                    │              ▼                 │   │
│                    │  ┌─────────────────────────┐  │   │
│                    │  │ 3. Fitness 평가         │  │   │
│                    │  │    → StateVectorSim     │  │   │
│                    │  └───────────┬─────────────┘  │   │
│                    │              │                 │   │
│                    └──────────────┼─────────────────┘   │
│                                   │                      │
│                                   ▼                      │
│                          OptimizedCircuit                │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

---

## 🔌 인터페이스 정의 (수정)

### 1. DriftScan (유지)
```rust
pub trait DriftScanner {
    fn scan(&mut self, qubit_id: usize) -> Result<NoiseVector>;
    fn is_anomaly(&self, noise: &NoiseVector) -> bool;
}

// Input
struct ScanConfig {
    qubit_id: usize,
    sample_count: usize,      // default: 1000
    interval_sec: u64,        // default: 300
    threshold_sigma: f64,     // default: 3.0
}

// Output
struct NoiseVector {
    t1_mean: f64,
    t1_std: f64,
    t2_mean: f64,
    t2_std: f64,
    drift_rate: f64,
    timestamp: u64,
}
```

### 2. GateReorder (수정 - LiveRewirer 내부 모듈로)
```rust
// LiveRewirer 내부에서만 사용
pub(crate) trait GateReorderOp {
    fn find_commuting_pairs(&self, circuit: &CircuitGenome) -> Vec<(usize, usize)>;
    fn generate_reorderings(&self, circuit: &CircuitGenome, max_variants: usize) -> Vec<CircuitGenome>;
}

// Input: CircuitGenome (내부 호출)
// Output: Vec<CircuitGenome> (내부 전달)
```

### 3. LiveRewirer (수정 - 통합 인터페이스)
```rust
pub trait LiveRewirer {
    fn load(&mut self, circuit: CircuitGenome) -> Result<()>;
    fn optimize(&mut self, noise: &NoiseVector, max_variants: usize) -> Result<CircuitGenome>;
}

// Input
struct RewireInput {
    circuit: CircuitGenome,
    noise_vector: NoiseVector,
    max_variants: usize,       // default: 10
    fitness_threshold: f64,    // default: 0.9
}

// Output
struct RewireOutput {
    optimized: CircuitGenome,
    fitness_score: f64,
    variants_evaluated: usize,
}
```

### 4. StateVectorSim (신규)
```rust
pub trait QuantumSimulator {
    fn execute(&mut self, circuit: &CircuitGenome) -> Result<StateVector>;
    fn measure(&mut self, shots: usize) -> Result<MeasureResult>;
    fn fidelity(&self, target: &StateVector) -> f64;
}

// Performance Target: <50ms for 10 qubits
```

---

## 🧱 핵심 타입 정의 (확장)

### Gate 열거형 (12개로 확장)
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Gate {
    // Single-qubit gates (9개)
    H(usize),                    // Hadamard
    X(usize),                    // Pauli-X
    Y(usize),                    // Pauli-Y
    Z(usize),                    // Pauli-Z
    S(usize),                    // Phase (√Z)
    T(usize),                    // π/8 gate
    Rx(usize, f64),              // Rotation-X
    Ry(usize, f64),              // Rotation-Y
    Rz(usize, f64),              // Rotation-Z
    
    // Two-qubit gates (3개)
    CNOT(usize, usize),          // Controlled-NOT
    CZ(usize, usize),            // Controlled-Z
    SWAP(usize, usize),          // SWAP
    
    // Measurement (1개)
    Measure(usize),              // Measurement
}

impl Gate {
    /// 게이트가 작용하는 큐비트 인덱스 반환
    pub fn qubits(&self) -> Vec<usize> {
        match self {
            Gate::H(q) | Gate::X(q) | Gate::Y(q) | Gate::Z(q) |
            Gate::S(q) | Gate::T(q) | Gate::Rx(q, _) | 
            Gate::Ry(q, _) | Gate::Rz(q, _) | Gate::Measure(q) => vec![*q],
            Gate::CNOT(c, t) | Gate::CZ(c, t) | Gate::SWAP(c, t) => vec![*c, *t],
        }
    }
    
    /// 두 게이트가 교환 가능한지 확인
    pub fn commutes_with(&self, other: &Gate) -> bool {
        let q1 = self.qubits();
        let q2 = other.qubits();
        // 큐비트가 겹치지 않으면 교환 가능
        q1.iter().all(|q| !q2.contains(q))
    }
}
```

### NoiseVector (확장)
```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NoiseVector {
    // T1/T2 통계
    pub t1_mean: f64,
    pub t1_std: f64,
    pub t2_mean: f64,
    pub t2_std: f64,
    
    // 드리프트 정보
    pub drift_rate: f64,
    pub burst_count: usize,
    
    // 메타데이터
    pub qubit_id: usize,
    pub timestamp: u64,
    pub sample_count: usize,
}

impl NoiseVector {
    pub fn new(qubit_id: usize) -> Self {
        Self {
            qubit_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ..Default::default()
        }
    }
    
    /// 이상 상태 여부 확인 (3σ 기준)
    pub fn is_anomaly(&self, threshold_sigma: f64) -> bool {
        self.drift_rate > threshold_sigma * self.t1_std ||
        self.burst_count > 0
    }
}
```

### CircuitGenome (확장)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitGenome {
    pub num_qubits: usize,
    pub gates: Vec<Gate>,
    pub metadata: CircuitMetadata,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CircuitMetadata {
    pub species_id: Option<String>,
    pub generation: usize,
    pub fitness_score: f64,
    pub parent_id: Option<String>,
}

impl CircuitGenome {
    pub fn new(num_qubits: usize) -> Self {
        Self {
            num_qubits,
            gates: Vec::new(),
            metadata: CircuitMetadata::default(),
        }
    }
    
    pub fn add_gate(&mut self, gate: Gate) -> Result<()> {
        // 큐비트 범위 검증
        for q in gate.qubits() {
            if q >= self.num_qubits {
                return Err(QnsError::InvalidQubit(q, self.num_qubits));
            }
        }
        self.gates.push(gate);
        Ok(())
    }
    
    pub fn depth(&self) -> usize {
        // 간단한 깊이 계산 (동시 실행 가능 게이트 고려)
        if self.gates.is_empty() {
            return 0;
        }
        
        let mut qubit_depths = vec![0usize; self.num_qubits];
        for gate in &self.gates {
            let qs = gate.qubits();
            let max_depth = qs.iter().map(|&q| qubit_depths[q]).max().unwrap_or(0);
            for &q in &qs {
                qubit_depths[q] = max_depth + 1;
            }
        }
        qubit_depths.into_iter().max().unwrap_or(0)
    }
}
```

### QnsError (통합 에러)
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QnsError {
    // Profiler errors
    #[error("Profiler error: {0}")]
    Profiler(String),
    
    #[error("Invalid qubit index {0}, max is {1}")]
    InvalidQubit(usize, usize),
    
    #[error("Anomaly detected: drift_rate={0:.4}")]
    AnomalyDetected(f64),
    
    // Rewire errors
    #[error("Rewire error: {0}")]
    Rewire(String),
    
    #[error("No circuit loaded")]
    NoCircuitLoaded,
    
    #[error("No valid variants found")]
    NoValidVariants,
    
    // Simulator errors
    #[error("Simulator error: {0}")]
    Simulator(String),
    
    #[error("State vector dimension mismatch")]
    DimensionMismatch,
    
    // I/O errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result 타입 alias
pub type Result<T> = std::result::Result<T, QnsError>;
```

---

## 📁 프로젝트 구조 (단일화 - 워크스페이스)

```
qns-mvp/
├── Cargo.toml                      # 워크스페이스 루트
├── Cargo.lock
├── README.md
├── LICENSE                         # MIT OR Apache-2.0
├── .gitignore
├── .rustfmt.toml
├── clippy.toml
│
├── crates/
│   ├── qns_core/                   # 핵심 타입
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types/
│   │       │   ├── mod.rs
│   │       │   ├── noise_vector.rs
│   │       │   ├── circuit_genome.rs
│   │       │   ├── gate.rs
│   │       │   └── hardware_profile.rs
│   │       ├── error.rs
│   │       ├── config.rs
│   │       └── prelude.rs
│   │
│   ├── qns_profiler/               # 노이즈 프로파일러
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── drift_scan/
│   │       │   ├── mod.rs
│   │       │   ├── scanner.rs
│   │       │   ├── measure.rs
│   │       │   └── anomaly.rs
│   │       └── burst_detector/
│   │           └── mod.rs
│   │
│   ├── qns_rewire/                 # 회로 재배선
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── gate_reorder/
│   │       │   ├── mod.rs
│   │       │   ├── commuting.rs
│   │       │   └── permute.rs
│   │       ├── live_rewirer/
│   │       │   ├── mod.rs
│   │       │   ├── engine.rs
│   │       │   ├── dag.rs
│   │       │   └── selector.rs
│   │       └── validator/
│   │           └── mod.rs
│   │
│   ├── qns_simulator/              # 양자 시뮬레이터
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── traits.rs
│   │       ├── state_vector/
│   │       │   ├── mod.rs
│   │       │   ├── simulator.rs
│   │       │   └── measure.rs
│   │       └── noise/
│   │           ├── mod.rs
│   │           └── models.rs
│   │
│   └── qns_cli/                    # CLI
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           └── commands/
│               ├── mod.rs
│               ├── profile.rs
│               └── rewire.rs
│
├── tests/                          # 통합 테스트
│   ├── pipeline_test.rs
│   └── e2e_test.rs
│
├── benches/                        # 벤치마크
│   └── performance.rs
│
└── examples/                       # 예제
    ├── basic_usage.rs
    └── optimize_circuit.rs
```

---

## 📦 워크스페이스 Cargo.toml (수정)

```toml
[workspace]
members = [
    "crates/qns_core",
    "crates/qns_profiler",
    "crates/qns_rewire",
    "crates/qns_simulator",
    "crates/qns_cli",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Jung Wook Yang <sadpig70@gmail.com>"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/qns-ai/qns-mvp"
rust-version = "1.75"

[workspace.dependencies]
# 내부 크레이트
qns_core = { path = "crates/qns_core" }
qns_profiler = { path = "crates/qns_profiler" }
qns_rewire = { path = "crates/qns_rewire" }
qns_simulator = { path = "crates/qns_simulator" }

# 비동기 런타임
tokio = { version = "1.35", features = ["full"] }

# 직렬화
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 에러 처리
anyhow = "1.0"
thiserror = "1.0"

# 수학 및 과학
ndarray = "0.15"
num-complex = "0.4"
rand = "0.8"
rand_distr = "0.4"

# 병렬 처리 (추가됨)
rayon = "1.10"

# CLI
clap = { version = "4.4", features = ["derive"] }

# 로깅
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# 테스트/벤치마크
criterion = "0.5"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

---

## 📊 통계 요약 (v2.0)

### 노드 현황

| 모듈 | 노드 수 | 레벨 | 상태 |
|------|---------|------|------|
| QNS_Core | 24 | 4 | 🆕 신규 |
| DriftScan | 13 | 4 | ✅ 완료 |
| BurstDetector | 4 | 3 | 🔜 설계중 |
| GateReorder | 8 | 4 | ✅ 완료 |
| LiveRewirer | 12 | 4 | ✅ 완료 |
| CircuitValidator | 3 | 3 | 🔜 설계중 |
| StateVectorSim | 12 | 4 | 🔜 설계중 |
| NoiseModel | 4 | 3 | 🔜 설계중 |
| **총합** | **80** | **4** | - |

### v1.1 대비 변경

| 항목 | v1.1 | v2.0 | 변경 |
|------|------|------|------|
| 총 노드 수 | 147 | 80 | -67 (원자화 통합) |
| 완료 노드 | 43 | 33 | -10 (재정의) |
| 최대 깊이 | 4 | 4 | 유지 |
| 크레이트 수 | 6 | 5 | -1 (qns_species MVP 제외) |

---

## 🎯 성능 목표 (명확화)

| 모듈 | 목표 | 측정 조건 |
|------|------|-----------|
| DriftScan | <10ms | 1000 shots |
| GateReorder | <20ms | 20 gates |
| LiveRewirer | <100ms | 10 variants |
| StateVectorSim | <50ms | 10 qubits |
| **전체 파이프라인** | <200ms | end-to-end |

---

## ✅ 검증 체크리스트

### 구조 검증
- [x] Gantree 루트에 QNS_Core 포함
- [x] 데이터 흐름 단방향 (순환 의존성 제거)
- [x] 모든 노드 4레벨 이내
- [x] 프로젝트 구조 단일화 (워크스페이스)

### 타입 검증
- [x] Gate enum 12개 정의
- [x] Result<T> 타입 alias 정의
- [x] QnsError 통합 에러 정의
- [x] NoiseVector 메타데이터 확장

### 의존성 검증
- [x] rayon 추가
- [x] thiserror 추가
- [x] 버전 고정

---

## 🚀 다음 단계

### 즉시 실행 가능
1. `cargo new --lib qns-mvp` 실행
2. 워크스페이스 Cargo.toml 생성
3. crates/ 디렉토리 구조 생성
4. qns_core 구현 시작

### Phase 1 목표 (Week 1)
- [ ] 워크스페이스 설정 완료
- [ ] qns_core 타입 구현
- [ ] 단위 테스트 작성

---

**작성일**: 2025-11-26  
**검증 상태**: ✅ 구조 검증 완료  
**다음 액션**: Rust 프로젝트 생성
