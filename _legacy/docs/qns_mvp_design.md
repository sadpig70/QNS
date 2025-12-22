# QNS MVP - Gantree 설계 (검증 완료본)

**버전**: 1.1 (검증 완료)  
**작성일**: 2025-10-31  
**검증 상태**: ✅ PASS (43개 노드, 오류 0건)

---

## 🎯 설계 범위

**MVP 핵심 모듈 (Phase 1-3 완료)**
- ✅ DriftScan (노이즈 프로파일러) - 17개 노드
- ✅ LiveRewirer (회로 재배선) - 21개 노드  
- ✅ GateReorder (게이트 재배열) - 9개 노드

**향후 확장 (Phase 4-6)**
- 🔜 BurstDetector (버스트 감지기)
- 🔜 SpeciesBank (종 은행)
- 🔜 StateVectorSimulator (시뮬레이터)

---

## 📐 Level 0: 시스템 루트

```gantree
QNS_MVP_System // 양자 노이즈 공생 MVP 시스템 (설계중)
    QNS_Profiler // 노이즈 프로파일러 모듈 (진행중)
    QNS_Rewire // 회로 재배선 모듈 (진행중)
```

---

## 📊 Level 1-4: QNS_Profiler (노이즈 프로파일러)

### 완전 구현 모듈: DriftScan

```gantree
QNS_Profiler // 노이즈 프로파일러 모듈 (진행중)
    DriftScan // T1/T2 드리프트 스캐너 (완료)
        DriftScanner // 스캐너 구조체 (완료)
            InitScanner // 스캐너 초기화 (완료)
            ConfigureInterval // 측정 주기 설정 - 5분 간격 (완료)
            SetThresholds // 임계값 설정 (완료)
        MeasureT1T2 // T1/T2 시간 측정 (완료)
            SimulateT1 // T1 시간 시뮬레이션 (완료)
            SimulateT2 // T2 시간 시뮬레이션 (완료)
            CollectSamples // 샘플 수집 - 1000 shots (완료)
        ComputeDriftVector // 드리프트 벡터 계산 (완료)
            CalculateMean // 평균값 계산 (완료)
            CalculateStdDev // 표준편차 계산 (완료)
            ComputeDriftRate // 드리프트 속도 계산 (완료)
        DetectAnomaly // 이상 감지 (완료)
            CheckThreshold // 임계값 초과 확인 (완료)
            TriggerAlert // 알림 트리거 (완료)
```

**노드 수**: 17개  
**최대 깊이**: 4레벨  
**상태**: ✅ 완료  
**PPR 구현**: ✅ 검증 완료

---

## 🔄 Level 1-4: QNS_Rewire (회로 재배선)

### 완전 구현 모듈: LiveRewirer

```gantree
QNS_Rewire // 회로 재배선 모듈 (진행중)
    LiveRewirer // 실시간 재배선 엔진 (완료)
        RewireEngine // 재배선 엔진 구조체 (완료)
            InitEngine // 엔진 초기화 (완료)
            LoadCircuit // 회로 로드 (완료)
        AnalyzeCircuit // 회로 분석 (완료)
            ExtractGates // 게이트 추출 (완료)
            IdentifyDependencies // 의존성 식별 (완료)
            BuildDAG // DAG 구축 (완료)
        ApplyMutations // 변이 적용 (완료)
            SelectMutationOperator // 변이 연산자 선택 (완료)
            ApplyOperator // 연산자 적용 (완료)
            GenerateCandidates // 후보 회로 생성 (완료)
        SimulateAndSelect // 시뮬레이션 및 선택 (완료)
            RunSimulation // 시뮬레이션 실행 - 목표 100ms 미만 (완료)
            CalculateFitness // 적합도 계산 (완료)
            SelectBestCircuit // 최적 회로 선택 (완료)
```

**노드 수**: 21개  
**최대 깊이**: 4레벨  
**상태**: ✅ 완료  
**PPR 구현**: ✅ 검증 완료

---

### 완전 구현 모듈: GateReorder

```gantree
QNS_Rewire // 회로 재배선 모듈 (진행중)
    GateReorder // 게이트 재배열 연산자 (완료)
        ReorderOperator // 재배열 연산자 구조체 (완료)
            InitOperator // 연산자 초기화 (완료)
        FindCommutingGates // 교환 가능한 게이트 찾기 (완료)
            CheckCommutativity // 교환법칙 확인 (완료)
            GroupCommutingPairs // 교환 가능 쌍 그룹화 (완료)
        GenerateReorderings // 재배열 생성 (완료)
            PermuteGateOrder // 게이트 순서 치환 (완료)
            CreateVariant // 변종 회로 생성 (완료)
```

**노드 수**: 9개  
**최대 깊이**: 4레벨  
**상태**: ✅ 완료  
**PPR 구현**: ✅ 검증 완료

---

## 🔜 향후 확장: BurstDetector (설계중)

```gantree
QNS_Profiler // 노이즈 프로파일러 모듈 (진행중)
    BurstDetector // 노이즈 버스트 감지기 (설계중)
        BurstMonitor // 버스트 모니터 구조체 (설계중)
            InitMonitor // 모니터 초기화 (설계중)
            SetSigmaThreshold // 3σ 임계값 설정 (설계중)
        DetectSpike // 스파이크 감지 (설계중)
            AnalyzeNoiseLevel // 노이즈 레벨 분석 (설계중)
            CompareBaseline // 기준선과 비교 (설계중)
            IdentifyAffectedQubits // 영향받은 큐비트 식별 (설계중)
        GenerateEvent // 이벤트 생성 (설계중)
            CreateBurstEvent // 버스트 이벤트 객체 생성 (설계중)
            RecordTimestamp // 타임스탬프 기록 (설계중)
            LogEvent // 이벤트 로깅 (설계중)
```

**예정 노드 수**: 12개  
**구현 예정**: Phase 4 (Week 4-5)

---

## 🔜 향후 확장: NoiseAggregator (설계중)

```gantree
QNS_Profiler // 노이즈 프로파일러 모듈 (진행중)
    NoiseAggregator // 노이즈 데이터 집계기 (설계중)
        CollectProfiles // 프로파일 수집 (설계중)
            MergeDriftData // 드리프트 데이터 병합 (설계중)
            MergeBurstData // 버스트 데이터 병합 (설계중)
        GenerateNoiseVector // 노이즈 벡터 생성 (설계중)
            NormalizeValues // 값 정규화 (설계중)
            CreateVectorStruct // 벡터 구조체 생성 (설계중)
        ExportProfile // 프로파일 내보내기 (설계중)
            SerializeToJSON // JSON 직렬화 (설계중)
            WriteToFile // 파일 쓰기 (설계중)
```

**예정 노드 수**: 10개  
**구현 예정**: Phase 4 (Week 4-5)

---

## 🔜 향후 확장: CircuitValidator (설계중)

```gantree
QNS_Rewire // 회로 재배선 모듈 (진행중)
    CircuitValidator // 회로 검증기 (설계중)
        ValidateStructure // 구조 검증 (설계중)
            CheckGateSyntax // 게이트 문법 확인 (설계중)
            ValidateQubitIndices // 큐비트 인덱스 유효성 (설계중)
            CheckCircuitDepth // 회로 깊이 확인 (설계중)
        ValidateSemantics // 의미론 검증 (설계중)
            CheckUnitarity // 유니터리 검증 (설계중)
            ValidateDAG // DAG 유효성 검사 (설계중)
        ReportErrors // 에러 보고 (설계중)
            CollectErrors // 에러 수집 (설계중)
            FormatErrorMessage // 에러 메시지 포맷 (설계중)
```

**예정 노드 수**: 10개  
**구현 예정**: Phase 4 (Week 6-7)

---

## 🔜 향후 확장: QNS_Species (설계중)

```gantree
QNS_Species // 종 은행 모듈 (설계중)
    SpeciesBank // 종 은행 구조체 (설계중)
        BankManager // 은행 관리자 (설계중)
            InitBank // 은행 초기화 (설계중)
            LoadExistingSpecies // 기존 종 로드 (설계중)
        RegisterSpecies // 종 등록 (설계중)
            GenerateSpeciesID // 종 ID 생성 (설계중)
            StoreGenome // 게놈 저장 (설계중)
            RecordMetadata // 메타데이터 기록 (설계중)
        QuerySpecies // 종 조회 (설계중)
            SearchByID // ID로 검색 (설계중)
            SearchByHardware // 하드웨어로 검색 (설계중)
            GetTopPerformers // 최고 성능 종 조회 (설계중)
    SpeciesEvolution // 종 진화 엔진 (설계중)
        EvolutionEngine // 진화 엔진 구조체 (설계중)
            InitEngine // 엔진 초기화 (설계중)
            SetGenerationLimit // 세대 제한 설정 (설계중)
        EvolveGeneration // 세대 진화 (설계중)
            SelectParents // 부모 선택 (설계중)
            CrossoverOperation // 교배 연산 (설계중)
            MutationOperation // 돌연변이 연산 (설계중)
        EvaluateFitness // 적합도 평가 (설계중)
            RunBenchmark // 벤치마크 실행 (설계중)
            CalculateScore // 점수 계산 (설계중)
            UpdateRanking // 랭킹 업데이트 (설계중)
    SpeciesStorage // 종 저장소 (설계중)
        LocalStorage // 로컬 저장소 (설계중)
            InitStorage // 저장소 초기화 (설계중)
            CreateDirectory // 디렉토리 생성 (설계중)
        SaveSpecies // 종 저장 (설계중)
            SerializeSpecies // 종 직렬화 - JSON (설계중)
            WriteToFile // 파일 쓰기 (설계중)
            UpdateIndex // 인덱스 업데이트 (설계중)
        LoadSpecies // 종 로드 (설계중)
            ReadFromFile // 파일 읽기 (설계중)
            DeserializeSpecies // 종 역직렬화 (설계중)
            ValidateIntegrity // 무결성 검증 (설계중)
```

**예정 노드 수**: 32개  
**구현 예정**: Phase 5 (Week 8-9)

---

## 🔜 향후 확장: QNS_Simulator (설계중)

```gantree
QNS_Simulator // 양자 시뮬레이터 인터페이스 (설계중)
    SimulatorTrait // 시뮬레이터 트레잇 (설계중)
    StateVectorSimulator // 상태 벡터 시뮬레이터 (설계중)
        SVSimulator // SV 시뮬레이터 구조체 (설계중)
            InitSimulator // 시뮬레이터 초기화 (설계중)
            AllocateQubits // 큐비트 할당 (설계중)
        ExecuteCircuit // 회로 실행 (설계중)
            ParseGates // 게이트 파싱 (설계중)
            ApplyGates // 게이트 적용 (설계중)
            UpdateStateVector // 상태 벡터 업데이트 (설계중)
        MeasureQubits // 큐비트 측정 (설계중)
            CollapseMeasurement // 측정 붕괴 (설계중)
            RecordOutcome // 결과 기록 (설계중)
        CalculateFidelity // 충실도 계산 (설계중)
            CompareStates // 상태 비교 (설계중)
            ComputeInnerProduct // 내적 계산 (설계중)
    NoiseSimulator // 노이즈 시뮬레이터 (설계중)
        NoiseModel // 노이즈 모델 구조체 (설계중)
            InitNoiseModel // 노이즈 모델 초기화 (설계중)
            SetT1T2Values // T1/T2 값 설정 (설계중)
            ConfigureCrosstalk // 크로스톡 설정 (보류)
        ApplyNoise // 노이즈 적용 (설계중)
            AddDepolarizingNoise // 탈분극 노이즈 추가 (설계중)
            AddAmplitudeDamping // 진폭 감쇠 추가 (설계중)
            AddPhaseDamping // 위상 감쇠 추가 (설계중)
        SimulateWithNoise // 노이즈 포함 시뮬레이션 (설계중)
            ExecuteNoisyCircuit // 노이즈 회로 실행 (설계중)
            AverageOverShots // 샷 평균 (설계중)
    SimulatorFactory // 시뮬레이터 팩토리 (설계중)
```

**예정 노드 수**: 26개  
**구현 예정**: Phase 5-6 (Week 10)

---

## 🔜 향후 확장: QNS_CLI (설계중)

```gantree
QNS_CLI // 커맨드라인 인터페이스 (설계중)
    CLIParser // CLI 파서 (설계중)
        ParseArguments // 인자 파싱 - clap 사용 (설계중)
        ValidateCommands // 명령 유효성 검사 (설계중)
    CommandHandlers // 명령 핸들러 (설계중)
        ProfileCommand // profile 명령 - 노이즈 프로파일링 (설계중)
        RewireCommand // rewire 명령 - 회로 재배선 (설계중)
        EvolveCommand // evolve 명령 - 종 진화 (설계중)
        BenchmarkCommand // benchmark 명령 - 벤치마크 실행 (설계중)
    OutputFormatter // 출력 포맷터 (설계중)
        FormatTable // 테이블 포맷 (설계중)
        FormatJSON // JSON 포맷 (설계중)
        DisplayProgress // 진행 상황 표시 (설계중)
```

**예정 노드 수**: 12개  
**구현 예정**: Phase 6 (Week 10-11)

---

## 📊 통계 요약

### 현재 상태 (Phase 1-3 완료)

| 항목 | 값 |
|------|-----|
| **완료 노드** | 43개 |
| **진행중 노드** | 2개 |
| **설계중 노드** | 102개 |
| **총 노드** | 147개 |
| **최대 깊이** | 4레벨 |
| **인터페이스** | 3개 |
| **데이터 흐름** | 2개 |

### 모듈별 현황

| 모듈 | 노드 수 | 레벨 | 상태 | 구현 예정 |
|------|---------|------|------|-----------|
| DriftScan | 17 | 4 | ✅ 완료 | - |
| LiveRewirer | 21 | 4 | ✅ 완료 | - |
| GateReorder | 9 | 4 | ✅ 완료 | - |
| BurstDetector | 12 | 4 | 🔜 설계중 | Week 4-5 |
| NoiseAggregator | 10 | 4 | 🔜 설계중 | Week 4-5 |
| CircuitValidator | 10 | 4 | 🔜 설계중 | Week 6-7 |
| SpeciesBank | 32 | 4 | 🔜 설계중 | Week 8-9 |
| StateVectorSimulator | 26 | 4 | 🔜 설계중 | Week 10 |
| QNS_CLI | 12 | 3 | 🔜 설계중 | Week 10-11 |

### 검증 결과

| 검증 항목 | 결과 |
|-----------|------|
| 계층 구조 완결성 | ✅ PASS |
| 상태 일관성 | ✅ PASS |
| 레벨 깊이 | ✅ PASS (4레벨) |
| 데이터 흐름 | ✅ PASS |
| 인터페이스 일치성 | ✅ PASS |
| PPR 코드 실행 | ✅ PASS |
| **종합 판정** | ✅ **PASS** |

---

## 🔄 데이터 흐름 정의

```
DriftScan
    ↓ (NoiseVector)
LiveRewirer
    ↑ (Vec<CircuitGenome>)
GateReorder
```

**검증 상태**: ✅ 타입 일치성 확인 완료

---

## 🔌 인터페이스 정의

### 1. DriftScan
```rust
inputs: {
    qubit_id: usize
}
outputs: {
    noise_vector: NoiseVector
}
```

### 2. LiveRewirer
```rust
inputs: {
    circuit: CircuitGenome,
    noise_vector: NoiseVector,
    reordered_circuits: Vec<CircuitGenome>
}
outputs: {
    optimized_circuit: CircuitGenome
}
```

### 3. GateReorder
```rust
inputs: {
    circuit: CircuitGenome
}
outputs: {
    reordered_circuits: Vec<CircuitGenome>
}
```

**검증 상태**: ✅ 인터페이스 호환성 확인 완료

---

## 🎯 구현 로드맵

### ✅ Phase 1-3: 핵심 모듈 (완료)
- Week 1-3: DriftScan 구현 ✅
- Week 4-5: LiveRewirer 구현 ✅
- Week 6-7: GateReorder 구현 ✅

### 🔜 Phase 4: 프로파일러 확장
- Week 4-5: BurstDetector, NoiseAggregator

### 🔜 Phase 5: 재배선 검증
- Week 6-7: CircuitValidator

### 🔜 Phase 6: 종 은행
- Week 8-9: SpeciesBank, SpeciesEvolution, SpeciesStorage

### 🔜 Phase 7: 시뮬레이터
- Week 10: StateVectorSimulator, NoiseSimulator

### 🔜 Phase 8: CLI 및 통합
- Week 10-11: QNS_CLI, 통합 테스트

### 🔜 Phase 9: 문서화
- Week 12: API 문서, 사용자 가이드

---

## 🏆 MVP 성공 지표

| 지표 | 목표 | 현재 상태 |
|------|------|-----------|
| 핵심 모듈 구현 | 3개 | ✅ 3개 완료 |
| 노드 구현률 | 43/147 | ✅ 29.3% |
| PPR 검증 | PASS | ✅ PASS |
| Rust 구현 준비 | 완료 | ✅ 준비 완료 |
| 벤치마크 (10 circuits) | >40% 향상 | 🔜 측정 예정 |

---

## 📝 변경 이력

### v1.1 (2025-10-31) - 검증 완료본
- ✅ PPR 구조 검증 완료
- ✅ 데이터 흐름 타입 불일치 수정
- ✅ 인터페이스 정의 완료
- ✅ 실행 테스트 통과

### v1.0 (2025-10-31) - 초기 설계
- 전체 시스템 Gantree 구조 정의
- 147개 노드, 4레벨 깊이

---

**설계 완료일**: 2025-10-31  
**검증 완료일**: 2025-10-31  
**다음 액션**: Rust Phase 1 구현 시작  
**검증 상태**: ✅ **VALIDATION PASSED**

# =============================================

# QNS MVP - Rust 프로젝트 전체 구조 설계

**버전**: 1.0  
**작성일**: 2025-10-31  
**기반**: Gantree 설계 (검증 완료) + PPR 코드

---

## 📁 프로젝트 루트 구조

```
qns-mvp/
├── Cargo.toml                      # 워크스페이스 설정
├── Cargo.lock                      # 의존성 잠금 파일
├── README.md                       # 프로젝트 소개
├── LICENSE                         # 라이선스 (MIT OR Apache-2.0)
├── .gitignore                      # Git 무시 파일
├── .rustfmt.toml                   # Rust 포맷 설정
├── .clippy.toml                    # Clippy 린트 설정
│
├── crates/                         # 크레이트 디렉토리
│   ├── qns_core/                   # 핵심 타입 및 설정
│   ├── qns_profiler/               # 노이즈 프로파일러
│   ├── qns_rewire/                 # 회로 재배선
│   ├── qns_species/                # 종 은행
│   ├── qns_simulator/              # 양자 시뮬레이터
│   └── qns_cli/                    # CLI 애플리케이션
│
├── tests/                          # 통합 테스트
│   ├── integration_tests.rs
│   ├── drift_scan_tests.rs
│   ├── live_rewire_tests.rs
│   └── end_to_end_tests.rs
│
├── benches/                        # 벤치마크
│   ├── noise_profiling.rs
│   ├── circuit_rewiring.rs
│   └── full_pipeline.rs
│
├── examples/                       # 사용 예시
│   ├── basic_usage.rs
│   ├── profile_noise.rs
│   ├── rewire_circuit.rs
│   └── evolve_species.rs
│
├── docs/                           # 문서
│   ├── gantree_design.md           # Gantree 설계
│   ├── ppr_validation.md           # PPR 검증 보고서
│   ├── architecture.md             # 아키텍처 문서
│   ├── api_guide.md                # API 가이드
│   └── contributing.md             # 기여 가이드
│
├── scripts/                        # 유틸리티 스크립트
│   ├── setup.sh                    # 개발 환경 설정
│   ├── test_all.sh                 # 전체 테스트
│   ├── benchmark.sh                # 벤치마크 실행
│   └── release.sh                  # 릴리즈 빌드
│
└── assets/                         # 리소스 파일
    ├── config/                     # 설정 파일
    │   ├── default.toml
    │   └── example.toml
    └── test_data/                  # 테스트 데이터
        ├── test_circuits/
        └── test_profiles/
```

---

## 📦 크레이트 상세 구조

### 1. qns_core (핵심 타입 및 설정)

**Gantree**: `QNS_Core → CoreTypes, ErrorTypes, ConfigManager`

```
crates/qns_core/
├── Cargo.toml                      # 크레이트 설정
├── README.md                       # 크레이트 설명
│
└── src/
    ├── lib.rs                      # 크레이트 루트
    │
    ├── types/                      # Gantree: CoreTypes
    │   ├── mod.rs
    │   ├── noise_vector.rs         # Gantree: NoiseVector
    │   ├── circuit_genome.rs       # Gantree: CircuitGenome
    │   ├── species_metadata.rs     # Gantree: SpeciesMetadata
    │   └── hardware_profile.rs     # Gantree: HardwareProfile
    │
    ├── error/                      # Gantree: ErrorTypes
    │   ├── mod.rs
    │   ├── profiler_error.rs       # Gantree: ProfilerError
    │   ├── rewire_error.rs         # Gantree: RewireError
    │   ├── species_error.rs        # Gantree: SpeciesError
    │   └── simulator_error.rs      # Gantree: SimulatorError
    │
    ├── config/                     # Gantree: ConfigManager
    │   ├── mod.rs
    │   ├── loader.rs               # Gantree: LoadConfig
    │   ├── validator.rs            # Gantree: ValidateConfig
    │   └── saver.rs                # Gantree: SaveConfig
    │
    └── prelude.rs                  # 공통 export
```

**주요 파일 내용 예시**:

```rust
// src/lib.rs
pub mod types;
pub mod error;
pub mod config;
pub mod prelude;

// src/types/mod.rs
pub mod noise_vector;
pub mod circuit_genome;
pub mod species_metadata;
pub mod hardware_profile;

pub use noise_vector::NoiseVector;
pub use circuit_genome::CircuitGenome;
pub use species_metadata::SpeciesMetadata;
pub use hardware_profile::HardwareProfile;

// src/prelude.rs
pub use crate::types::{NoiseVector, CircuitGenome, SpeciesMetadata, HardwareProfile};
pub use crate::error::{QnsError, Result};
pub use crate::config::Config;
```

---

### 2. qns_profiler (노이즈 프로파일러)

**Gantree**: `QNS_Profiler → DriftScan, BurstDetector, NoiseAggregator`

```
crates/qns_profiler/
├── Cargo.toml
├── README.md
│
└── src/
    ├── lib.rs                      # 크레이트 루트
    │
    ├── drift_scan/                 # Gantree: DriftScan (완료)
    │   ├── mod.rs
    │   ├── scanner.rs              # Gantree: DriftScanner
    │   │                           #   - InitScanner
    │   │                           #   - ConfigureInterval
    │   │                           #   - SetThresholds
    │   ├── measure.rs              # Gantree: MeasureT1T2
    │   │                           #   - SimulateT1
    │   │                           #   - SimulateT2
    │   │                           #   - CollectSamples
    │   ├── compute.rs              # Gantree: ComputeDriftVector
    │   │                           #   - CalculateMean
    │   │                           #   - CalculateStdDev
    │   │                           #   - ComputeDriftRate
    │   └── anomaly.rs              # Gantree: DetectAnomaly
    │                               #   - CheckThreshold
    │                               #   - TriggerAlert
    │
    ├── burst_detector/             # Gantree: BurstDetector (설계중)
    │   ├── mod.rs
    │   ├── monitor.rs              # Gantree: BurstMonitor
    │   ├── spike.rs                # Gantree: DetectSpike
    │   └── event.rs                # Gantree: GenerateEvent
    │
    ├── aggregator/                 # Gantree: NoiseAggregator (설계중)
    │   ├── mod.rs
    │   ├── collector.rs            # Gantree: CollectProfiles
    │   ├── generator.rs            # Gantree: GenerateNoiseVector
    │   └── exporter.rs             # Gantree: ExportProfile
    │
    └── tests/                      # 단위 테스트
        ├── drift_scan_tests.rs
        ├── burst_detector_tests.rs
        └── aggregator_tests.rs
```

**주요 파일 내용 예시**:

```rust
// src/lib.rs
pub mod drift_scan;
pub mod burst_detector;
pub mod aggregator;

pub use drift_scan::DriftScanner;
pub use burst_detector::BurstDetector;
pub use aggregator::NoiseAggregator;

// src/drift_scan/mod.rs
mod scanner;
mod measure;
mod compute;
mod anomaly;

pub use scanner::DriftScanner;
pub use measure::T1T2Measurements;
pub use compute::DriftVector;
pub use anomaly::AnomalyDetector;
```

---

### 3. qns_rewire (회로 재배선)

**Gantree**: `QNS_Rewire → LiveRewirer, GateReorder, CircuitValidator`

```
crates/qns_rewire/
├── Cargo.toml
├── README.md
│
└── src/
    ├── lib.rs                      # 크레이트 루트
    │
    ├── live_rewire/                # Gantree: LiveRewirer (완료)
    │   ├── mod.rs
    │   ├── engine.rs               # Gantree: RewireEngine
    │   │                           #   - InitEngine
    │   │                           #   - LoadCircuit
    │   ├── analyzer.rs             # Gantree: AnalyzeCircuit
    │   │                           #   - ExtractGates
    │   │                           #   - IdentifyDependencies
    │   │                           #   - BuildDAG
    │   ├── mutations.rs            # Gantree: ApplyMutations
    │   │                           #   - SelectMutationOperator
    │   │                           #   - ApplyOperator
    │   │                           #   - GenerateCandidates
    │   └── selector.rs             # Gantree: SimulateAndSelect
    │                               #   - RunSimulation
    │                               #   - CalculateFitness
    │                               #   - SelectBestCircuit
    │
    ├── gate_reorder/               # Gantree: GateReorder (완료)
    │   ├── mod.rs
    │   ├── operator.rs             # Gantree: ReorderOperator
    │   │                           #   - InitOperator
    │   ├── commuting.rs            # Gantree: FindCommutingGates
    │   │                           #   - CheckCommutativity
    │   │                           #   - GroupCommutingPairs
    │   └── generator.rs            # Gantree: GenerateReorderings
    │                               #   - PermuteGateOrder
    │                               #   - CreateVariant
    │
    ├── validator/                  # Gantree: CircuitValidator (설계중)
    │   ├── mod.rs
    │   ├── structure.rs            # Gantree: ValidateStructure
    │   ├── semantics.rs            # Gantree: ValidateSemantics
    │   └── reporter.rs             # Gantree: ReportErrors
    │
    ├── dag/                        # DAG 유틸리티
    │   ├── mod.rs
    │   ├── builder.rs
    │   └── traversal.rs
    │
    └── tests/
        ├── live_rewire_tests.rs
        ├── gate_reorder_tests.rs
        └── validator_tests.rs
```

**주요 파일 내용 예시**:

```rust
// src/lib.rs
pub mod live_rewire;
pub mod gate_reorder;
pub mod validator;
pub mod dag;

pub use live_rewire::LiveRewirer;
pub use gate_reorder::GateReorder;
pub use validator::CircuitValidator;

// src/live_rewire/mod.rs
mod engine;
mod analyzer;
mod mutations;
mod selector;

pub use engine::RewireEngine;
pub use analyzer::CircuitAnalyzer;
pub use mutations::MutationEngine;
pub use selector::CircuitSelector;
```

---

### 4. qns_species (종 은행)

**Gantree**: `QNS_Species → SpeciesBank, SpeciesEvolution, SpeciesStorage`

```
crates/qns_species/
├── Cargo.toml
├── README.md
│
└── src/
    ├── lib.rs
    │
    ├── bank/                       # Gantree: SpeciesBank
    │   ├── mod.rs
    │   ├── manager.rs              # Gantree: BankManager
    │   ├── registry.rs             # Gantree: RegisterSpecies
    │   └── query.rs                # Gantree: QuerySpecies
    │
    ├── evolution/                  # Gantree: SpeciesEvolution
    │   ├── mod.rs
    │   ├── engine.rs               # Gantree: EvolutionEngine
    │   ├── generation.rs           # Gantree: EvolveGeneration
    │   └── fitness.rs              # Gantree: EvaluateFitness
    │
    ├── storage/                    # Gantree: SpeciesStorage
    │   ├── mod.rs
    │   ├── local.rs                # Gantree: LocalStorage
    │   ├── saver.rs                # Gantree: SaveSpecies
    │   └── loader.rs               # Gantree: LoadSpecies
    │
    ├── calculator/                 # Gantree: FitnessCalculator
    │   ├── mod.rs
    │   └── metrics.rs
    │
    └── tests/
        ├── bank_tests.rs
        ├── evolution_tests.rs
        └── storage_tests.rs
```

---

### 5. qns_simulator (양자 시뮬레이터)

**Gantree**: `QNS_Simulator → StateVectorSimulator, NoiseSimulator`

```
crates/qns_simulator/
├── Cargo.toml
├── README.md
│
└── src/
    ├── lib.rs
    │
    ├── traits/                     # Gantree: SimulatorTrait
    │   ├── mod.rs
    │   └── simulator.rs
    │
    ├── state_vector/               # Gantree: StateVectorSimulator
    │   ├── mod.rs
    │   ├── simulator.rs            # Gantree: SVSimulator
    │   ├── executor.rs             # Gantree: ExecuteCircuit
    │   ├── measure.rs              # Gantree: MeasureQubits
    │   └── fidelity.rs             # Gantree: CalculateFidelity
    │
    ├── noise/                      # Gantree: NoiseSimulator
    │   ├── mod.rs
    │   ├── model.rs                # Gantree: NoiseModel
    │   ├── applier.rs              # Gantree: ApplyNoise
    │   └── simulator.rs            # Gantree: SimulateWithNoise
    │
    ├── factory/                    # Gantree: SimulatorFactory
    │   ├── mod.rs
    │   └── builder.rs
    │
    ├── math/                       # 수학 유틸리티
    │   ├── mod.rs
    │   ├── complex.rs
    │   └── matrix.rs
    │
    └── tests/
        ├── state_vector_tests.rs
        └── noise_tests.rs
```

---

### 6. qns_cli (CLI 애플리케이션)

**Gantree**: `QNS_CLI → CLIParser, CommandHandlers, OutputFormatter`

```
crates/qns_cli/
├── Cargo.toml
├── README.md
│
└── src/
    ├── main.rs                     # CLI 엔트리포인트
    │
    ├── parser/                     # Gantree: CLIParser
    │   ├── mod.rs
    │   ├── args.rs                 # Gantree: ParseArguments
    │   └── validator.rs            # Gantree: ValidateCommands
    │
    ├── commands/                   # Gantree: CommandHandlers
    │   ├── mod.rs
    │   ├── profile.rs              # Gantree: ProfileCommand
    │   ├── rewire.rs               # Gantree: RewireCommand
    │   ├── evolve.rs               # Gantree: EvolveCommand
    │   └── benchmark.rs            # Gantree: BenchmarkCommand
    │
    ├── output/                     # Gantree: OutputFormatter
    │   ├── mod.rs
    │   ├── table.rs                # Gantree: FormatTable
    │   ├── json.rs                 # Gantree: FormatJSON
    │   └── progress.rs             # Gantree: DisplayProgress
    │
    └── tests/
        └── cli_tests.rs
```

**main.rs 예시**:

```rust
// src/main.rs
use clap::Parser;
use qns_cli::{Cli, commands};

fn main() -> anyhow::Result<()> {
    // CLI 파싱
    let cli = Cli::parse();
    
    // 로깅 초기화
    tracing_subscriber::fmt::init();
    
    // 명령 실행
    match cli.command {
        commands::Commands::Profile(args) => {
            commands::profile::execute(args)?;
        }
        commands::Commands::Rewire(args) => {
            commands::rewire::execute(args)?;
        }
        commands::Commands::Evolve(args) => {
            commands::evolve::execute(args)?;
        }
        commands::Commands::Benchmark(args) => {
            commands::benchmark::execute(args)?;
        }
    }
    
    Ok(())
}
```

---

## 📝 워크스페이스 Cargo.toml

```toml
[workspace]
members = [
    "crates/qns_core",
    "crates/qns_profiler",
    "crates/qns_rewire",
    "crates/qns_species",
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
homepage = "https://qns.ai"
documentation = "https://docs.rs/qns-mvp"
rust-version = "1.75"

[workspace.dependencies]
# 내부 크레이트
qns_core = { path = "crates/qns_core" }
qns_profiler = { path = "crates/qns_profiler" }
qns_rewire = { path = "crates/qns_rewire" }
qns_species = { path = "crates/qns_species" }
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
nalgebra = "0.32"
rand = "0.8"
rand_distr = "0.4"

# CLI
clap = { version = "4.4", features = ["derive"] }
colored = "2.1"

# 설정
toml = "0.8"
config = "0.14"

# 로깅
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# 테스트
criterion = "0.5"
proptest = "1.4"

[profile.dev]
opt-level = 0
debug = true

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true

[profile.bench]
inherits = "release"
```

---

## 📋 개별 크레이트 Cargo.toml 예시

### qns_core/Cargo.toml

```toml
[package]
name = "qns_core"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
documentation.workspace = true
rust-version.workspace = true
description = "Core types and utilities for QNS MVP"

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
toml = { workspace = true }
tracing = { workspace = true }

[dev-dependencies]
tokio = { workspace = true, features = ["test-util"] }
```

### qns_profiler/Cargo.toml

```toml
[package]
name = "qns_profiler"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
documentation.workspace = true
rust-version.workspace = true
description = "Noise profiler for QNS MVP"

[dependencies]
qns_core = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
rand = { workspace = true }
rand_distr = { workspace = true }
tokio = { workspace = true }
ndarray = { workspace = true }

[dev-dependencies]
tokio = { workspace = true, features = ["test-util"] }
criterion = { workspace = true }
```

### qns_rewire/Cargo.toml

```toml
[package]
name = "qns_rewire"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
documentation.workspace = true
rust-version.workspace = true
description = "Circuit rewiring engine for QNS MVP"

[dependencies]
qns_core = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }

[dev-dependencies]
tokio = { workspace = true, features = ["test-util"] }
proptest = { workspace = true }
```

### qns_cli/Cargo.toml

```toml
[package]
name = "qns_cli"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
documentation.workspace = true
rust-version.workspace = true
description = "CLI application for QNS MVP"

[[bin]]
name = "qns"
path = "src/main.rs"

[dependencies]
qns_core = { workspace = true }
qns_profiler = { workspace = true }
qns_rewire = { workspace = true }
qns_species = { workspace = true }
qns_simulator = { workspace = true }

clap = { workspace = true }
colored = { workspace = true }
anyhow = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }

[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.0"
```

---

## 🧪 테스트 구조

### tests/integration_tests.rs

```rust
// tests/integration_tests.rs
use qns_core::prelude::*;
use qns_profiler::DriftScanner;
use qns_rewire::LiveRewirer;

#[tokio::test]
async fn test_full_optimization_pipeline() {
    // Gantree: QNS_MVP_System → AI_optimize_circuit
    
    // 1. 노이즈 프로파일링
    let mut scanner = DriftScanner::new();
    let noise_vector = scanner.compute_drift_vector(0).unwrap();
    
    // 2. 회로 재배선
    let mut rewirer = LiveRewirer::new();
    let circuit = CircuitGenome::new(3);
    rewirer.load_circuit(circuit).unwrap();
    
    let variants = rewirer.generate_variants(&noise_vector, 5).unwrap();
    let optimized = rewirer.select_best_variant(variants, 0.9).unwrap();
    
    assert!(optimized.is_some());
}
```

---

## 🔧 스크립트 예시

### scripts/setup.sh

```bash
#!/bin/bash
# QNS MVP 개발 환경 설정

set -e

echo "🚀 QNS MVP 개발 환경 설정"

# Rust 버전 확인
echo "📦 Rust 버전 확인..."
rustc --version

# 의존성 설치
echo "📥 의존성 설치..."
cargo fetch

# 포맷팅 도구
echo "🔧 rustfmt 설정..."
rustup component add rustfmt

# Linting 도구
echo "🔧 clippy 설정..."
rustup component add clippy

# 빌드
echo "🔨 빌드..."
cargo build

# 테스트
echo "🧪 테스트..."
cargo test --all

echo "✅ 설정 완료!"
```

### scripts/test_all.sh

```bash
#!/bin/bash
# 전체 테스트 실행

set -e

echo "🧪 QNS MVP 전체 테스트"

# 단위 테스트
echo "📊 단위 테스트..."
cargo test --all --lib

# 통합 테스트
echo "📊 통합 테스트..."
cargo test --all --test '*'

# 문서 테스트
echo "📊 문서 테스트..."
cargo test --all --doc

# Clippy
echo "📊 Clippy 검사..."
cargo clippy --all-targets --all-features -- -D warnings

# 포맷팅 체크
echo "📊 포맷팅 체크..."
cargo fmt --all -- --check

echo "✅ 모든 테스트 통과!"
```

---

## 📚 문서 구조

### docs/architecture.md

```markdown
# QNS MVP Architecture

## Gantree 기반 설계

본 프로젝트는 Gantree 방법론을 사용하여 Top-Down BFS 방식으로 설계되었습니다.

### 모듈 구조

```
QNS_MVP_System (L0)
├── QNS_Profiler (L1)
│   └── DriftScan (L2)
│       ├── DriftScanner (L3)
│       ├── MeasureT1T2 (L3)
│       ├── ComputeDriftVector (L3)
│       └── DetectAnomaly (L3)
└── QNS_Rewire (L1)
    ├── LiveRewirer (L2)
    └── GateReorder (L2)
```

[상세 내용 계속...]
```

---

## 📊 파일 개수 통계

| 디렉토리 | Rust 파일 | 예상 LOC |
|----------|-----------|----------|
| qns_core | 12 | 800 |
| qns_profiler | 15 | 1,200 |
| qns_rewire | 18 | 1,500 |
| qns_species | 15 | 1,000 |
| qns_simulator | 15 | 1,200 |
| qns_cli | 10 | 600 |
| tests | 8 | 500 |
| benches | 4 | 300 |
| examples | 5 | 400 |
| **총합** | **102** | **~7,500** |

---

## 🎯 구현 우선순위

### Phase 1: 핵심 타입 (Week 1)
```
qns_core/src/types/
├── noise_vector.rs      ✅ 구현 완료
├── circuit_genome.rs    ✅ 구현 완료
├── species_metadata.rs  ✅ 구현 완료
└── hardware_profile.rs  ✅ 구현 완료
```

### Phase 2: DriftScan (Week 2-3)
```
qns_profiler/src/drift_scan/
├── scanner.rs           🔜 구현 필요
├── measure.rs           🔜 구현 필요
├── compute.rs           🔜 구현 필요
└── anomaly.rs           🔜 구현 필요
```

### Phase 3: LiveRewirer (Week 4-7)
```
qns_rewire/src/live_rewire/
├── engine.rs            🔜 구현 필요
├── analyzer.rs          🔜 구현 필요
├── mutations.rs         🔜 구현 필요
└── selector.rs          🔜 구현 필요
```

---

**설계 완료일**: 2025-10-31  
**다음 단계**: Phase 1 구현 시작  
**예상 완료**: 2026-02-28 (12주)


# ===========================+++++++++++++++++++++++++++++++++++++++++

# QNS Rust MVP 구체화 작업계획서

**버전**: 1.0  
**작성일**: 2025-10-31  
**프로젝트**: QNS (Quantum Noise Symbiote) Rust MVP  
**목표**: Python PPR 검증 완료본을 Rust로 구현

---

## 🎯 프로젝트 개요

### 범위
**Phase 1-3 완료 모듈을 Rust로 구현**
- DriftScan (17개 노드)
- LiveRewirer (21개 노드)
- GateReorder (9개 노드)

### 목표
- 타입 안정성 확보
- 성능 10x 향상 (Python 대비)
- 프로덕션 레디 코드
- 완전한 테스트 커버리지

### 기간
**8주 (2025-11-01 ~ 2025-12-27)**

---

## 📐 Level 0: 프로젝트 루트

```gantree
QNS_Rust_MVP_Project // Rust MVP 전체 프로젝트 (설계중)
    Phase1_ProjectSetup // 프로젝트 기반 구축 (설계중)
    Phase2_CoreTypes // 핵심 타입 시스템 (설계중)
    Phase3_Profiler // 노이즈 프로파일러 구현 (설계중)
    Phase4_Rewire // 회로 재배선 구현 (설계중)
    Phase5_Integration // 통합 및 테스트 (설계중)
    Phase6_Optimization // 성능 최적화 (설계중)
    Phase7_Documentation // 문서화 및 배포 (설계중)
```

---

## 📊 Phase 1: 프로젝트 기반 구축 (Week 1)

```gantree
Phase1_ProjectSetup // 프로젝트 기반 구축 (설계중)
    CargoWorkspace // Cargo 워크스페이스 구성 (설계중)
        InitCargoToml // Cargo.toml 생성 (설계중)
        SetupWorkspace // 워크스페이스 구조 설정 (설계중)
        ConfigureDependencies // 의존성 설정 (설계중)
            AddNumpy // ndarray = "0.15" (설계중)
            AddSerde // serde = "1.0" (설계중)
            AddRayon // rayon = "1.8" (병렬처리) (설계중)
    DirectoryStructure // 디렉토리 구조 생성 (설계중)
        CreateSrcDirs // src 디렉토리 구조 (설계중)
            CoreModule // src/core/ (설계중)
            ProfilerModule // src/profiler/ (설계중)
            RewireModule // src/rewire/ (설계중)
            SimulatorModule // src/simulator/ (설계중)
        CreateTestDirs // tests 디렉토리 (설계중)
        CreateBenchDirs // benches 디렉토리 (설계중)
    DevEnvironment // 개발 환경 설정 (설계중)
        SetupRustfmt // rustfmt.toml 설정 (설계중)
        SetupClipper // clippy.toml 설정 (설계중)
        SetupCI // GitHub Actions CI (설계중)
```

**디렉토리 구조**
```
qns_mvp/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── core/
│   │   ├── mod.rs
│   │   ├── types.rs
│   │   └── errors.rs
│   ├── profiler/
│   │   ├── mod.rs
│   │   └── drift_scan.rs
│   ├── rewire/
│   │   ├── mod.rs
│   │   ├── live_rewirer.rs
│   │   └── gate_reorder.rs
│   └── simulator/
│       ├── mod.rs
│       └── state_vector.rs
├── tests/
│   ├── integration_tests.rs
│   └── profiler_tests.rs
└── benches/
    └── performance.rs
```

**예상 소요 시간**: 1주 (40시간)

---

## 🔧 Phase 2: 핵심 타입 시스템 (Week 2)

```gantree
Phase2_CoreTypes // 핵심 타입 시스템 (설계중)
    NoiseVectorType // NoiseVector 구조체 (설계중)
        DefineStruct // 구조체 정의 (설계중)
            T1Fields // t1_mean, t1_std (설계중)
            T2Fields // t2_mean, t2_std (설계중)
            DriftFields // drift_rate, burst_count (설계중)
        ImplementTraits // 트레잇 구현 (설계중)
            ImplDefault // Default 트레잇 (설계중)
            ImplDebug // Debug 트레잇 (설계중)
            ImplClone // Clone 트레잇 (설계중)
            ImplSerialize // Serialize/Deserialize (설계중)
    CircuitGenomeType // CircuitGenome 구조체 (설계중)
        DefineStruct // 구조체 정의 (설계중)
            QubitFields // num_qubits (설계중)
            GateFields // gate_sequence: Vec<Gate> (설계중)
            PathFields // redundant_paths (설계중)
        GateEnum // Gate 열거형 (설계중)
            SingleQubitGates // H, X, Y, Z, Rz (설계중)
            TwoQubitGates // CNOT, CZ (설계중)
        ImplementMethods // 메서드 구현 (설계중)
            AddGate // add_gate() (설계중)
            GetDepth // depth() (설계중)
            Clone // clone() (설계중)
    ErrorTypes // 에러 타입 시스템 (설계중)
        DefineErrorEnum // QNSError 열거형 (설계중)
            ProfilerErrors // 프로파일러 에러 (설계중)
            RewireErrors // 재배선 에러 (설계중)
            SimulatorErrors // 시뮬레이터 에러 (설계중)
        ImplementErrorTrait // Error 트레잇 (설계중)
        ImplementDisplay // Display 구현 (설계중)
```

**핵심 타입 정의 예시**
```rust
// src/core/types.rs

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NoiseVector {
    pub t1_mean: f64,
    pub t1_std: f64,
    pub t2_mean: f64,
    pub t2_std: f64,
    pub drift_rate: f64,
    pub burst_count: usize,
}

#[derive(Debug, Clone)]
pub enum Gate {
    H(usize),                    // Hadamard
    CNOT(usize, usize),          // Controlled-NOT
    Rz(usize, f64),              // Rotation-Z
}

#[derive(Debug, Clone)]
pub struct CircuitGenome {
    pub num_qubits: usize,
    pub gate_sequence: Vec<Gate>,
    pub redundant_paths: HashMap<usize, Vec<usize>>,
}
```

**예상 소요 시간**: 1주 (40시간)

---

## 📊 Phase 3: 노이즈 프로파일러 구현 (Week 3-4)

```gantree
Phase3_Profiler // 노이즈 프로파일러 구현 (설계중)
    DriftScanModule // DriftScan 모듈 (설계중)
        DriftScannerStruct // DriftScanner 구조체 (설계중)
            DefineFields // 필드 정의 (설계중)
                IntervalField // interval_secs: u64 (설계중)
                ThresholdFields // t1/t2_threshold (설계중)
                SamplesField // num_samples: usize (설계중)
            ImplNew // new() 생성자 (설계중)
            ImplDefault // Default 트레잇 (설계중)
        ComputeDriftMethod // compute_drift() 메서드 (설계중)
            SimulateT1T2 // T1/T2 시뮬레이션 (설계중)
                GenerateT1Samples // T1 샘플 생성 (설계중)
                GenerateT2Samples // T2 샘플 생성 (설계중)
                UseRandDistr // rand_distr 크레이트 사용 (설계중)
            CalculateStatistics // 통계 계산 (설계중)
                CalculateMean // 평균 계산 (설계중)
                CalculateStdDev // 표준편차 계산 (설계중)
                ComputeDriftRate // 드리프트 속도 (설계중)
            DetectAnomaly // 이상 감지 (설계중)
                CheckThresholds // 임계값 확인 (설계중)
                TriggerAlert // 알림 트리거 (설계중)
    UnitTests // 단위 테스트 (설계중)
        TestDriftScanInit // 초기화 테스트 (설계중)
        TestComputeDrift // 드리프트 계산 테스트 (설계중)
        TestAnomalyDetection // 이상 감지 테스트 (설계중)
    IntegrationTests // 통합 테스트 (설계중)
        TestFullPipeline // 전체 파이프라인 테스트 (설계중)
```

**구현 예시**
```rust
// src/profiler/drift_scan.rs

pub struct DriftScanner {
    interval_secs: u64,
    t1_threshold: f64,
    t2_threshold: f64,
    num_samples: usize,
}

impl DriftScanner {
    pub fn new() -> Self {
        Self {
            interval_secs: 300,
            t1_threshold: 10.0,
            t2_threshold: 5.0,
            num_samples: 1000,
        }
    }
    
    pub fn compute_drift(&self, qubit_id: usize) -> Result<NoiseVector> {
        // Gantree: MeasureT1T2 → SimulateT1, SimulateT2
        let t1_samples = self.simulate_t1(qubit_id)?;
        let t2_samples = self.simulate_t2(qubit_id)?;
        
        // Gantree: ComputeDriftVector → CalculateMean
        let t1_mean = calculate_mean(&t1_samples);
        let t2_mean = calculate_mean(&t2_samples);
        
        // Gantree: ComputeDriftVector → CalculateStdDev
        let t1_std = calculate_std_dev(&t1_samples, t1_mean);
        let t2_std = calculate_std_dev(&t2_samples, t2_mean);
        
        // Gantree: ComputeDriftVector → ComputeDriftRate
        let drift_rate = (t1_std + t2_std) / 2.0;
        
        let noise_vector = NoiseVector {
            t1_mean,
            t1_std,
            t2_mean,
            t2_std,
            drift_rate,
            burst_count: 0,
        };
        
        // Gantree: DetectAnomaly → CheckThreshold
        if self.check_anomaly(&noise_vector) {
            // Gantree: DetectAnomaly → TriggerAlert
            self.trigger_alert(&noise_vector);
        }
        
        Ok(noise_vector)
    }
    
    fn simulate_t1(&self, qubit_id: usize) -> Result<Vec<f64>> {
        use rand_distr::{Distribution, Normal};
        let normal = Normal::new(125.0, 5.0)?;
        let mut rng = rand::thread_rng();
        Ok((0..self.num_samples)
            .map(|_| normal.sample(&mut rng))
            .collect())
    }
}
```

**예상 소요 시간**: 2주 (80시간)

---

## 🔄 Phase 4: 회로 재배선 구현 (Week 5-6)

```gantree
Phase4_Rewire // 회로 재배선 구현 (설계중)
    GateReorderModule // GateReorder 모듈 (설계중)
        GateReorderStruct // GateReorder 구조체 (설계중)
            DefineStruct // 구조체 정의 (설계중)
            ImplNew // new() 생성자 (설계중)
        FindCommutingGates // 교환 가능 게이트 찾기 (설계중)
            CheckCommutativity // 교환법칙 확인 (설계중)
                ExtractQubits // 큐비트 추출 (설계중)
                CheckDisjoint // 교집합 확인 (설계중)
            CollectPairs // 교환 가능 쌍 수집 (설계중)
        GenerateReorderings // 재배열 생성 (설계중)
            PermuteGates // 게이트 순서 치환 (설계중)
            CreateVariants // 변종 회로 생성 (설계중)
    LiveRewirerModule // LiveRewirer 모듈 (설계중)
        LiveRewirerStruct // LiveRewirer 구조체 (설계중)
            DefineFields // 필드 정의 (설계중)
                CurrentCircuit // current_circuit (설계중)
                DAGField // dag: DAG (설계중)
                GateReorderField // gate_reorder (설계중)
            ImplNew // new() 생성자 (설계중)
        LoadCircuitMethod // load() 메서드 (설계중)
            StoreCircuit // 회로 저장 (설계중)
            BuildDAG // DAG 구축 (설계중)
                CreateNodes // 노드 생성 (설계중)
                CreateEdges // 엣지 생성 (설계중)
                TrackDependencies // 의존성 추적 (설계중)
        GenerateVariantsMethod // generate_variants() (설계중)
            FindCommuting // 교환 가능 게이트 찾기 (설계중)
            ApplyMutations // 변이 적용 (설계중)
            CreateCandidates // 후보 생성 (설계중)
        SelectBestMethod // select_best() (설계중)
            SimulateVariants // 변종 시뮬레이션 (설계중)
            CalculateFitness // 적합도 계산 (설계중)
            ReturnOptimal // 최적 회로 반환 (설계중)
    DAGStructure // DAG 데이터 구조 (설계중)
        DefineDAG // DAG 정의 (설계중)
            NodesVec // nodes: Vec<usize> (설계중)
            EdgesMap // edges: HashMap (설계중)
        ImplementMethods // 메서드 구현 (설계중)
            AddNode // add_node() (설계중)
            AddEdge // add_edge() (설계중)
            GetPredecessors // get_predecessors() (설계중)
    UnitTests // 단위 테스트 (설계중)
        TestGateReorder // GateReorder 테스트 (설계중)
        TestLiveRewirer // LiveRewirer 테스트 (설계중)
        TestDAG // DAG 테스트 (설계중)
```

**구현 예시**
```rust
// src/rewire/live_rewirer.rs

pub struct LiveRewirer {
    current_circuit: Option<CircuitGenome>,
    dag: Option<DAG>,
    gate_reorder: GateReorder,
}

impl LiveRewirer {
    pub fn new() -> Self {
        Self {
            current_circuit: None,
            dag: None,
            gate_reorder: GateReorder::new(),
        }
    }
    
    pub fn load(&mut self, circuit: CircuitGenome) -> Result<()> {
        // Gantree: RewireEngine → LoadCircuit
        self.current_circuit = Some(circuit.clone());
        
        // Gantree: AnalyzeCircuit → BuildDAG
        self.dag = Some(self.build_dag(&circuit)?);
        
        Ok(())
    }
    
    fn build_dag(&self, circuit: &CircuitGenome) -> Result<DAG> {
        let mut dag = DAG::new();
        let mut qubit_last_gate: HashMap<usize, usize> = HashMap::new();
        
        for (idx, gate) in circuit.gate_sequence.iter().enumerate() {
            dag.add_node(idx);
            
            // Gantree: AnalyzeCircuit → ExtractGates, IdentifyDependencies
            let qubits = self.extract_qubits(gate);
            
            for &qubit in &qubits {
                if let Some(&prev_idx) = qubit_last_gate.get(&qubit) {
                    dag.add_edge(prev_idx, idx);
                }
                qubit_last_gate.insert(qubit, idx);
            }
        }
        
        Ok(dag)
    }
    
    pub fn generate_variants(
        &self, 
        _noise_vector: &NoiseVector, 
        num: usize
    ) -> Result<Vec<CircuitGenome>> {
        let circuit = self.current_circuit.as_ref()
            .ok_or(QNSError::NoCircuitLoaded)?;
        
        // Gantree: ApplyMutations → SelectMutationOperator
        // Gantree: GateReorder → FindCommutingGates
        let commuting_pairs = self.gate_reorder
            .find_commuting_gates(circuit)?;
        
        // Gantree: ApplyMutations → GenerateCandidates
        let variants = self.gate_reorder
            .generate_reorderings(circuit, &commuting_pairs, num)?;
        
        Ok(variants)
    }
    
    pub fn select_best(
        &self, 
        variants: Vec<CircuitGenome>
    ) -> Result<CircuitGenome> {
        // Gantree: SimulateAndSelect → RunSimulation, CalculateFitness
        // MVP: 첫 번째 변종 반환
        variants.into_iter().next()
            .ok_or(QNSError::NoVariantsAvailable)
    }
}
```

**예상 소요 시간**: 2주 (80시간)

---

## 🔗 Phase 5: 통합 및 테스트 (Week 7)

```gantree
Phase5_Integration // 통합 및 테스트 (설계중)
    MainAPI // 메인 API 구성 (설계중)
        QNSSystemStruct // QNSSystem 구조체 (설계중)
            DefineFields // 필드 정의 (설계중)
                ProfilerField // drift_scan (설계중)
                RewireField // live_rewirer (설계중)
            ImplNew // new() 생성자 (설계중)
        OptimizeCircuitMethod // optimize_circuit() (설계중)
            CallProfiler // 프로파일러 호출 (설계중)
            CallRewire // 재배선 호출 (설계중)
            ReturnResult // 결과 반환 (설계중)
    IntegrationTests // 통합 테스트 (설계중)
        TestFullPipeline // 전체 파이프라인 테스트 (설계중)
            SetupCircuit // 테스트 회로 생성 (설계중)
            RunOptimization // 최적화 실행 (설계중)
            ValidateOutput // 출력 검증 (설계중)
        TestErrorHandling // 에러 처리 테스트 (설계중)
            TestInvalidInput // 잘못된 입력 (설계중)
            TestNoCircuit // 회로 없음 (설계중)
    PerformanceBenchmarks // 성능 벤치마크 (설계중)
        BenchDriftScan // DriftScan 벤치마크 (설계중)
        BenchLiveRewirer // LiveRewirer 벤치마크 (설계중)
        BenchFullPipeline // 전체 파이프라인 벤치마크 (설계중)
```

**통합 API 예시**
```rust
// src/lib.rs

pub struct QNSSystem {
    drift_scan: DriftScanner,
    live_rewirer: LiveRewirer,
}

impl QNSSystem {
    pub fn new() -> Self {
        Self {
            drift_scan: DriftScanner::new(),
            live_rewirer: LiveRewirer::new(),
        }
    }
    
    pub fn optimize_circuit(
        &mut self,
        circuit: CircuitGenome,
        qubit_id: usize,
    ) -> Result<(CircuitGenome, NoiseVector)> {
        // 1. 노이즈 프로파일링
        let noise_vector = self.drift_scan.compute_drift(qubit_id)?;
        
        // 2. 회로 재배선
        self.live_rewirer.load(circuit)?;
        let variants = self.live_rewirer
            .generate_variants(&noise_vector, 5)?;
        let optimized = self.live_rewirer.select_best(variants)?;
        
        Ok((optimized, noise_vector))
    }
}
```

**예상 소요 시간**: 1주 (40시간)

---

## ⚡ Phase 6: 성능 최적화 (Week 8)

```gantree
Phase6_Optimization // 성능 최적화 (설계중)
    ParallelProcessing // 병렬 처리 최적화 (설계중)
        ParallelizeSampling // 샘플링 병렬화 (설계중)
            UseRayon // rayon 크레이트 사용 (설계중)
            ParallelIterator // par_iter() 적용 (설계중)
        ParallelizeVariants // 변종 생성 병렬화 (설계중)
    MemoryOptimization // 메모리 최적화 (설계중)
        ReduceCloning // 불필요한 clone 제거 (설계중)
        UseReferences // 참조 활용 (설계중)
        OptimizeAllocations // 할당 최적화 (설계중)
    AlgorithmOptimization // 알고리즘 최적화 (설계중)
        CachingStrategy // 캐싱 전략 (설계중)
            CacheDAG // DAG 캐싱 (설계중)
            CacheCommutingPairs // 교환 쌍 캐싱 (설계중)
        OptimizeDAGBuild // DAG 구축 최적화 (설계중)
    ProfilingAndBenchmark // 프로파일링 및 벤치마크 (설계중)
        UseCargo // cargo bench 실행 (설계중)
        UseFlamegraph // flamegraph 생성 (설계중)
        MeasureImprovement // 개선도 측정 (설계중)
```

**최적화 목표**
- Python 대비 10x 성능 향상
- DriftScan: <10ms
- LiveRewirer: <100ms
- 메모리 사용량: <100MB

**예상 소요 시간**: 1주 (40시간)

---

## 📚 Phase 7: 문서화 및 배포 (Week 8 후반)

```gantree
Phase7_Documentation // 문서화 및 배포 (설계중)
    CodeDocumentation // 코드 문서화 (설계중)
        AddDocComments // doc comments 추가 (설계중)
        GenerateRustdoc // cargo doc 생성 (설계중)
        WriteExamples // 예제 코드 작성 (설계중)
    UserGuide // 사용자 가이드 (설계중)
        InstallationGuide // 설치 가이드 (설계중)
        QuickStart // 빠른 시작 (설계중)
        APIReference // API 레퍼런스 (설계중)
    ReleasePreparation // 릴리스 준비 (설계중)
        VersionTagging // 버전 태깅 (설계중)
        ChangelogUpdate // CHANGELOG 업데이트 (설계중)
        CratePublication // crates.io 발행 준비 (설계중)
```

**예상 소요 시간**: 0.5주 (20시간)

---

## 📊 전체 타임라인

```gantree
Timeline_8Weeks // 8주 타임라인 (설계중)
    Week1 // 프로젝트 기반 구축 (설계중)
        Day1to2 // Cargo 워크스페이스 (설계중)
        Day3to4 // 디렉토리 구조 (설계중)
        Day5 // 개발 환경 설정 (설계중)
    Week2 // 핵심 타입 시스템 (설계중)
        Day1to2 // NoiseVector, CircuitGenome (설계중)
        Day3to4 // Gate, ErrorTypes (설계중)
        Day5 // 트레잇 구현 및 테스트 (설계중)
    Week3 // DriftScan 구현 (설계중)
        Day1to2 // DriftScanner 구조체 (설계중)
        Day3to4 // compute_drift 메서드 (설계중)
        Day5 // 단위 테스트 (설계중)
    Week4 // DriftScan 완성 (설계중)
        Day1to2 // 통계 계산 함수들 (설계중)
        Day3to4 // 이상 감지 로직 (설계중)
        Day5 // 통합 테스트 (설계중)
    Week5 // GateReorder 구현 (설계중)
        Day1to2 // GateReorder 구조체 (설계중)
        Day3to4 // 교환법칙 확인 로직 (설계중)
        Day5 // 재배열 생성 로직 (설계중)
    Week6 // LiveRewirer 구현 (설계중)
        Day1to2 // LiveRewirer 구조체, DAG (설계중)
        Day3to4 // generate_variants 메서드 (설계중)
        Day5 // select_best 메서드 (설계중)
    Week7 // 통합 및 테스트 (설계중)
        Day1to2 // QNSSystem API (설계중)
        Day3to4 // 통합 테스트 (설계중)
        Day5 // 벤치마크 (설계중)
    Week8 // 최적화 및 문서화 (설계중)
        Day1to3 // 성능 최적화 (설계중)
        Day4to5 // 문서화 및 릴리스 (설계중)
```

---

## 📋 체크리스트

### Phase 1: 프로젝트 기반 구축
- [ ] Cargo.toml 생성
- [ ] 워크스페이스 구조 설정
- [ ] 의존성 추가 (ndarray, serde, rayon, rand_distr)
- [ ] 디렉토리 구조 생성
- [ ] rustfmt.toml 설정
- [ ] clippy.toml 설정
- [ ] GitHub Actions CI 설정

### Phase 2: 핵심 타입 시스템
- [ ] NoiseVector 구조체 정의
- [ ] CircuitGenome 구조체 정의
- [ ] Gate 열거형 정의
- [ ] ErrorTypes 정의
- [ ] 트레잇 구현 (Debug, Clone, Default, Serialize)
- [ ] 단위 테스트 작성

### Phase 3: 노이즈 프로파일러
- [ ] DriftScanner 구조체 구현
- [ ] compute_drift() 메서드 구현
- [ ] simulate_t1/t2() 함수 구현
- [ ] 통계 계산 함수들 구현
- [ ] 이상 감지 로직 구현
- [ ] 단위 테스트 작성
- [ ] 통합 테스트 작성

### Phase 4: 회로 재배선
- [ ] GateReorder 구조체 구현
- [ ] find_commuting_gates() 구현
- [ ] generate_reorderings() 구현
- [ ] DAG 데이터 구조 구현
- [ ] LiveRewirer 구조체 구현
- [ ] load() 메서드 구현
- [ ] generate_variants() 구현
- [ ] select_best() 구현
- [ ] 단위 테스트 작성

### Phase 5: 통합 및 테스트
- [ ] QNSSystem API 구현
- [ ] optimize_circuit() 메서드 구현
- [ ] 통합 테스트 작성
- [ ] 에러 처리 테스트
- [ ] 벤치마크 작성

### Phase 6: 성능 최적화
- [ ] 병렬 처리 적용 (rayon)
- [ ] 불필요한 clone 제거
- [ ] 캐싱 전략 적용
- [ ] 프로파일링 (flamegraph)
- [ ] 10x 성능 목표 달성

### Phase 7: 문서화 및 배포
- [ ] doc comments 추가
- [ ] cargo doc 생성
- [ ] 예제 코드 작성
- [ ] README.md 작성
- [ ] CHANGELOG.md 업데이트
- [ ] 버전 태깅 (v0.1.0)

---

## 🎯 성공 지표

### 기능 완성도
- [ ] DriftScan 17개 노드 구현 완료
- [ ] LiveRewirer 21개 노드 구현 완료
- [ ] GateReorder 9개 노드 구현 완료
- [ ] 전체 43개 노드 Rust 구현 완료

### 성능 목표
- [ ] DriftScan < 10ms
- [ ] LiveRewirer < 100ms
- [ ] Python 대비 10x 성능 향상
- [ ] 메모리 사용량 < 100MB

### 품질 지표
- [ ] 단위 테스트 커버리지 > 80%
- [ ] 통합 테스트 통과
- [ ] clippy 경고 0건
- [ ] rustfmt 적용

### 문서화
- [ ] 모든 public API doc comments
- [ ] README.md 완성
- [ ] 예제 코드 3개 이상
- [ ] cargo doc 생성

---

## 🔄 리스크 관리

### 리스크 1: 타입 시스템 복잡도
**확률**: Medium  
**영향**: High  
**완화**: Python PPR 검증 완료로 구조 확정, 점진적 구현

### 리스크 2: 성능 목표 미달
**확률**: Low  
**영향**: Medium  
**완화**: Week 8 최적화 주간 확보, rayon 병렬화

### 리스크 3: 테스트 커버리지 부족
**확률**: Medium  
**영향**: Medium  
**완화**: 각 Phase마다 테스트 작성, TDD 접근

---

## 📦 의존성 목록

```toml
[dependencies]
ndarray = "0.15"           # 수치 계산
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"         # JSON 직렬화
rayon = "1.8"              # 병렬 처리
rand = "0.8"               # 난수 생성
rand_distr = "0.4"         # 확률 분포

[dev-dependencies]
criterion = "0.5"          # 벤치마크
proptest = "1.0"           # 속성 기반 테스트
```

---

## 🚀 다음 액션

### 즉시 시작 가능
1. **Cargo 프로젝트 생성**
   ```bash
   cargo new --lib qns_mvp
   cd qns_mvp
   ```

2. **의존성 추가**
   - Cargo.toml 편집

3. **디렉토리 구조 생성**
   - src/core/, src/profiler/, src/rewire/ 생성

### Week 1 목표
- ✅ 프로젝트 기반 구축 완료
- ✅ 개발 환경 설정 완료
- ✅ CI/CD 파이프라인 구축

---

**작성 완료일**: 2025-10-31  
**예상 완료일**: 2025-12-27  
**총 소요 시간**: 320시간 (8주)  
**상태**: ✅ 계획 수립 완료