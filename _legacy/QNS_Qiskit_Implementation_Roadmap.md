# QNS-Qiskit Integration Implementation Roadmap

> **설계 기반**: QNS_Qiskit_Integration_Gantree.md  
> **방법론**: PPR/Gantree 프레임워크, Top-Down BFS  
> **진행 방식**: 원자화 노드 단위 구현 → 통합 → 검증

---

## 🎯 전체 로드맵 개요

```
QNS_Qiskit_Implementation_Roadmap // 전체 구현 로드맵 (진행중)
    Sprint1_Phase1L1 // Phase1 L1: Python-Rust Bridge (설계중)
    Sprint2_Phase1L2 // Phase1 L2: Calibration Integration (설계중)
    Sprint3_Phase1L3 // Phase1 L3: Aer Simulation (설계중)
    Sprint4_Phase1L4 // Phase1 L4: CLI Integration (설계중)
    Sprint5_Phase2L1 // Phase2 L1: IBM Runtime Integration (설계중)
    Sprint6_Phase2L2 // Phase2 L2: Validation Scripts (설계중)
```

---

## 📅 Sprint 1: Python-Rust Bridge (Week 1)

**목표**: QNS CircuitGenome ↔ Qiskit QuantumCircuit 상호 변환 및 PyO3 바인딩

### Gantree 작업 구조

```
Sprint1_Phase1L1 // Python-Rust Bridge 구축 (진행중)
    Task1_1_ProjectSetup // 프로젝트 환경 설정 (설계중)
        InstallQiskit // Qiskit 패키지 설치 (설계중)
            CreateRequirementsTxt // requirements.txt 생성 (설계중)
            RunPipInstall // pip install 실행 (설계중)
        SetupPythonModule // Python 모듈 구조 생성 (설계중)
            CreateQiskitBridgePy // qiskit_bridge.py 파일 생성 (설계중)
            CreateInitPy // __init__.py 생성 (설계중)
    
    Task1_2_CircuitConverter // 회로 변환기 구현 (설계중)
        ImplementQNSToQiskit // QNS → Qiskit 변환 (설계중)
            ParseQNSGates // QNS 게이트 파싱 함수 (설계중)
            MapToQiskitGates // Qiskit 게이트 매핑 함수 (설계중)
            BuildQuantumCircuit // QuantumCircuit 생성 함수 (설계중)
        WriteUnitTests // 단위 테스트 작성 (설계중)
            TestBellState // Bell state 변환 테스트 (설계중)
            TestGHZState // GHZ state 변환 테스트 (설계중)
            TestAllGateTypes // 모든 게이트 타입 테스트 (설계중)
    
    Task1_3_PyO3Bindings // PyO3 바인딩 구현 (설계중)
        UpdateLibRs // lib.rs 업데이트 (설계중)
            AddCircuitConversionFunction // circuit 변환 함수 추가 (설계중)
            AddPyModuleExport // Python 모듈 export (설계중)
        BuildAndTest // 빌드 및 테스트 (설계중)
            CargoMaturinBuild // maturin build 실행 (설계중)
            ImportTestInPython // Python에서 import 테스트 (설계중)
    
    Task1_4_IntegrationTest // 통합 테스트 (설계중)
        EndToEndTest // E2E 테스트 (설계중)
            CreateTestCircuitInRust // Rust에서 테스트 회로 생성 (설계중)
            ConvertToPython // Python으로 변환 (설계중)
            RunQiskitSimulator // Qiskit 시뮬레이터 실행 (설계중)
            ValidateResults // 결과 검증 (설계중)
```

### 체크리스트

- [ ] Qiskit 설치 완료 (`requirements.txt`, `pip install`)
- [ ] `qns_python/src/qiskit_bridge.py` 파일 생성
- [ ] `ParseQNSGates`, `MapToQiskitGates`, `BuildQuantumCircuit` 함수 구현
- [ ] 단위 테스트 3개 이상 작성 및 통과
- [ ] `lib.rs`에 PyO3 바인딩 추가
- [ ] `maturin build` 성공
- [ ] Python에서 `from qns_python import convert_circuit` 성공
- [ ] Bell state E2E 테스트 통과

**예상 소요 시간**: 4-6시간  
**완료 조건**: Bell state가 QNS → Qiskit 변환되어 Aer에서 실행됨

---

## 📅 Sprint 2: Calibration Integration (Week 1-2)

**목표**: IBM 백엔드 캘리브레이션 데이터 → Qiskit NoiseModel → QNS NoiseVector

### Gantree 작업 구조

```
Sprint2_Phase1L2 // Calibration Integration (진행중)
    Task2_1_IBMConnection // IBM 백엔드 연결 (설계중)
        SetupQiskitRuntime // Qiskit Runtime 설정 (설계중)
            LoadAPIKey // .env에서 API Key 로딩 (설계중)
            InitializeService // QiskitRuntimeService 초기화 (설계중)
        SelectBackend // 백엔드 선택 로직 (설계중)
            ListBackends // 사용 가능 백엔드 조회 (설계중)
            FilterByQubits // 큐비트 수로 필터링 (설계중)
            ChooseBackend // 백엔드 선택 (설계중)
    
    Task2_2_CalibrationFetch // 캘리브레이션 데이터 조회 (설계중)
        FetchProperties // backend.properties() 호출 (설계중)
        ParseT1T2 // T1/T2 데이터 파싱 (설계중)
            ExtractQubitT1 // 각 큐비트 T1 추출 (설계중)
            ExtractQubitT2 // 각 큐비트 T2 추출 (설계중)
            ValidateT2Constraint // T2 ≤ 2*T1 검증 (설계중)
        ParseGateErrors // 게이트 에러율 파싱 (설계중)
            Extract1QErrors // 단일 큐비트 게이트 에러 (설계중)
            Extract2QErrors // 2큐비트 게이트 에러 (설계중)
        ParseReadoutErrors // 측정 에러율 파싱 (설계중)
    
    Task2_3_NoiseModelBuilder // NoiseModel 생성 (설계중)
        CreateNoiseModel // NoiseModel 객체 생성 (설계중)
        AddT1T2Errors // T1/T2 에러 추가 (설계중)
            CreateAmplitudeDamping // Amplitude damping 채널 (설계중)
            CreatePhaseDamping // Phase damping 채널 (설계중)
        AddGateErrors // 게이트 에러 추가 (설계중)
            AddDepolarizingError // Depolarizing 에러 (설계중)
        AddReadoutErrors // 측정 에러 추가 (설계중)
    
    Task2_4_NoiseVectorAdapter // QNS NoiseVector 변환 (설계중)
        MapToNoiseVector // Calibration → NoiseVector (설계중)
        ValidateNoiseVector // NoiseVector 검증 (설계중)
        ExportToPyO3 // PyO3로 Rust에 전달 (설계중)
```

### 체크리스트

- [ ] `.env` 파일에 `QISKIT_IBM_TOKEN` 설정
- [ ] `QiskitRuntimeService` 인증 성공
- [ ] `ibm_fez` (또는 다른 백엔드) 선택 성공
- [ ] `backend.properties()` 호출 및 T1/T2 데이터 추출
- [ ] NoiseModel 생성 성공 (T1/T2/gate errors 포함)
- [ ] NoiseVector 변환 함수 구현
- [ ] PyO3 바인딩으로 Rust에 전달 성공

**예상 소요 시간**: 3-4시간  
**완료 조건**: `ibm_fez` 캘리브레이션으로 NoiseModel 생성 및 NoiseVector 변환

---

## 📅 Sprint 3: Aer Simulation (Week 2)

**목표**: Aer 노이즈 시뮬레이션 및 QNS 효과 검증

### Gantree 작업 구조

```
Sprint3_Phase1L3 // Aer Simulation (진행중)
    Task3_1_SimulatorSetup // 시뮬레이터 설정 (설계중)
        CreateAerSimulator // AerSimulator 인스턴스 생성 (설계중)
        AttachNoiseModel // NoiseModel 연결 (설계중)
    
    Task3_2_CircuitExecution // 회로 실행 (설계중)
        PrepareCircuit // 회로 준비 (설계중)
            AddMeasurements // 측정 게이트 추가 (설계중)
            ValidateCircuit // 회로 유효성 검사 (설계중)
        RunSimulation // 시뮬레이션 실행 (설계중)
            ExecuteWithShots // shots 수만큼 실행 (설계중)
            CollectResults // 결과 수집 (설계중)
        ParseResults // 결과 파싱 (설계중)
            ExtractCounts // 카운트 추출 (설계중)
            CalculateFidelity // 피델리티 계산 (설계중)
    
    Task3_3_ComparativeAnalysis // QNS vs 비최적화 비교 (설계중)
        RunIdentityMapping // Identity 매핑 실행 (설계중)
            CreateIdentityCircuit // Identity 회로 생성 (설계중)
            ExecuteIdentity // 실행 및 측정 (설계중)
        RunQNSOptimized // QNS 최적화 실행 (설계중)
            OptimizeCircuit // QNS 최적화 적용 (설계중)
            ExecuteOptimized // 실행 및 측정 (설계중)
        ComputeDelta // 피델리티 차이 계산 (설계중)
            CalculateFidelityGain // 향상률 계산 (설계중)
            GenerateComparisonReport // 비교 리포트 생성 (설계중)
    
    Task3_4_Validation // 검증 (설계중)
        VerifyFidelityRange // 피델리티 범위 검증 (0-1) (설계중)
        CheckQNSImprovement // QNS 향상 확인 (설계중)
        SaveResults // 결과 저장 (설계중)
```

### 체크리스트

- [ ] AerSimulator + NoiseModel 생성 성공
- [ ] 회로에 측정 게이트 자동 추가
- [ ] 시뮬레이션 실행 및 카운트 추출
- [ ] 피델리티 계산 로직 구현
- [ ] Identity vs. QNS 비교 실행
- [ ] QNS 최적화 회로가 +5~10% 피델리티 향상 확인
- [ ] 결과를 JSON 파일로 저장

**예상 소요 시간**: 5-6시간  
**완료 조건**: Aer 시뮬레이션에서 QNS가 Identity 대비 피델리티 향상 확인

---

## 📅 Sprint 4: CLI Integration (Week 2-3)

**목표**: CLI `--backend` 옵션 추가 및 Qiskit 백엔드 선택 지원

### Gantree 작업 구조

```
Sprint4_Phase1L4 // CLI Integration (진행중)
    Task4_1_CLIExtension // CLI 확장 (설계중)
        AddBackendOption // --backend 옵션 추가 (설계중)
            UpdateMainRs // main.rs 수정 (설계중)
            AddBackendEnum // BackendType enum 정의 (설계중)
        AddIBMBackendOption // --ibm-backend 옵션 추가 (설계중)
    
    Task4_2_QiskitRunner // Qiskit 실행 모듈 (설계중)
        CreateRunnerModule // qiskit_runner.rs 생성 (설계중)
        ImplementBackendSelector // 백엔드 선택 로직 (설계중)
            SelectSimulator // Simulator 모드 (설계중)
            SelectAerNoisy // AerNoisy 모드 (설계중)
            SelectIBM // IBM Hardware 모드 (보류) (설계중)
        ImplementExecutor // 실행 엔진 (설계중)
            CallPythonBridge // Python 브리지 호출 (설계중)
            HandleResults // 결과 처리 (설계중)
    
    Task4_3_IntegrationTest // 통합 테스트 (설계중)
        TestSimulatorMode // Simulator 모드 테스트 (설계중)
        TestAerNoisyMode // AerNoisy 모드 테스트 (설계중)
        TestCLIOutput // CLI 출력 검증 (설계중)
```

### 체크리스트

- [ ] `main.rs`에 `--backend` / `--ibm-backend` 옵션 추가
- [ ] `BackendType` enum 정의 (Simulator, AerNoisy, IBM)
- [ ] `qiskit_runner.rs` 모듈 생성
- [ ] 백엔드별 실행 분기 로직 구현
- [ ] `qns run --backend aer-noisy --ibm-backend ibm_fez circuit.qasm` 실행 성공
- [ ] 결과가 JSON/Text 형식으로 출력

**예상 소요 시간**: 2-3시간  
**완료 조건**: CLI에서 `aer-noisy` 백엔드 선택 및 실행 성공

---

## 📅 Sprint 5: IBM Runtime Integration (Week 3-4)

**목표**: 실제 IBM Quantum 하드웨어 Job 제출 및 결과 수신

### Gantree 작업 구조

```
Sprint5_Phase2L1 // IBM Runtime Integration (진행중)
    Task5_1_SessionManagement // Session 관리 (설계중)
        CreateSession // Session 생성 (설계중)
        SelectHardwareBackend // 하드웨어 백엔드 선택 (설계중)
    
    Task5_2_JobSubmission // Job 제출 (설계중)
        TranspileCircuit // 회로 트랜스파일 (설계중)
        PrepareJob // Job 준비 (설계중)
        SubmitToQueue // 큐에 제출 (설계중)
    
    Task5_3_JobMonitoring // Job 모니터링 (설계중)
        CheckQueuePosition // 큐 위치 확인 (설계중)
        WaitForCompletion // 완료 대기 (설계중)
        RetrieveResults // 결과 수신 (설계중)
    
    Task5_4_ErrorHandling // 에러 처리 (설계중)
        HandleTimeout // 타임아웃 처리 (설계중)
        HandleJobFailure // Job 실패 처리 (설계중)
        RetryLogic // 재시도 로직 (설계중)
```

### 체크리스트

- [ ] Qiskit Runtime Session 생성 성공
- [ ] 회로 트랜스파일 (백엔드 coupling map 반영)
- [ ] Job 제출 및 Job ID 수신
- [ ] 큐 상태 모니터링 로직 구현
- [ ] Job 완료 후 결과 수신 성공
- [ ] 에러 발생 시 재시도 로직 작동

**예상 소요 시간**: 4-5시간  
**완료 조건**: Bell state가 실제 IBM 하드웨어에서 실행되고 결과 수신

---

## 📅 Sprint 6: Validation Scripts (Week 4)

**목표**: 벤치마크 회로 세트 실행 및 통계 분석

### Gantree 작업 구조

```
Sprint6_Phase2L2 // Validation Scripts (진행중)
    Task6_1_BenchmarkCircuits // 벤치마크 회로 준비 (설계중)
        CreateBellState // Bell state 회로 (설계중)
        CreateGHZState // GHZ state 회로 (설계중)
        CreateQFT // QFT 회로 (설계중)
    
    Task6_2_ComparativeExperiment // 비교 실험 (설계중)
        RunQiskitBaseline // Qiskit 기본 트랜스파일 (설계중)
        RunQNSOptimized // QNS 최적화 (설계중)
        ExecuteOnHardware // 하드웨어 실행 (설계중)
    
    Task6_3_StatisticalAnalysis // 통계 분석 (설계중)
        CalculateMeanFidelity // 평균 피델리티 (설계중)
        PerformTTest // t-test (설계중)
        ComputeCohenD // Cohen's d (설계중)
    
    Task6_4_ResultCollection // 결과 수집 (설계중)
        SaveJSON // JSON 저장 (설계중)
        SaveCSV // CSV 저장 (설계중)
        GeneratePlots // 시각화 (설계중)
```

### 체크리스트

- [ ] 벤치마크 회로 3개 이상 준비
- [ ] Qiskit vs. QNS 비교 실험 스크립트 작성
- [ ] 각 회로를 3회 이상 반복 실행
- [ ] 통계 분석 (mean, std, t-test, effect size)
- [ ] 결과를 JSON/CSV로 저장
- [ ] matplotlib로 시각화 생성

**예상 소요 시간**: 6-8시간  
**완료 조건**: 5개 이상 회로에서 통계적으로 유의한 피델리티 향상 확인

---

## 📊 전체 타임라인

| Sprint | 주차 | 예상 시간 | 누적 시간 |
|--------|------|-----------|----------|
| Sprint 1 | Week 1 | 4-6h | 4-6h |
| Sprint 2 | Week 1-2 | 3-4h | 7-10h |
| Sprint 3 | Week 2 | 5-6h | 12-16h |
| Sprint 4 | Week 2-3 | 2-3h | 14-19h |
| Sprint 5 | Week 3-4 | 4-5h | 18-24h |
| Sprint 6 | Week 4 | 6-8h | 24-32h |

**총 예상 시간**: 24-32시간 (파트타임 기준 4주)

---

## ✅ 각 Sprint 완료 기준

### Sprint 완료 체크리스트

- [ ] 모든 원자화 노드가 구현됨
- [ ] 단위 테스트 통과율 100%
- [ ] 통합 테스트 성공
- [ ] 문서 업데이트 (코드 주석, README)
- [ ] Git commit 및 tag 생성

### 최종 완료 조건

- [ ] Phase 1 시뮬레이션에서 +5~10% 피델리티 향상
- [ ] Phase 2 하드웨어에서 통계적 유의성 (p < 0.05)
- [ ] 5개 이상 벤치마크 회로 검증 완료
- [ ] 결과 시각화 및 리포트 생성

---

*설계 프레임워크: PPR/Gantree V4*  
*로드맵 버전: 1.0*  
*최종 업데이트: 2025-12-17*
