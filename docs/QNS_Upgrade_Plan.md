# QNS arXiv 논문 보강 작업 명세서

**작성일:** 2025년 12월 21일  
**목적:** arXiv 논문 게재를 위한 시뮬레이션 기반 벤치마크 보강

---

## 1. 수학적 형식화 보강

> ✅ **완료** - 상세 내용은 [`QNS_Mathematical_Formalization.md`](./QNS_Mathematical_Formalization.md) 참조

### 1.1 LiveRewirer 최적화 목표 함수 정의

**목표 함수:**

$$
C^* = \arg\max_{C' \in \mathcal{V}(C)} \hat{F}(C', \mathbf{n}(t))
$$

여기서:

- $C$: 원본 회로
- $\mathcal{V}(C) = \{ C' : U_{C'} = U_C \}$: 수학적 동등 회로 변종 집합
- $\mathbf{n}(t) = (T_1, T_2, \boldsymbol{\epsilon})^T$: 시간 의존 노이즈 프로파일 벡터
- $\hat{F}$: 예상 충실도 추정 함수

### 1.2 Fidelity 추정 모델 수식화

$$
\hat{F}(C, \mathbf{n}) = \underbrace{(1 - \epsilon_{1q})^{n_{1q}} \cdot (1 - \epsilon_{2q})^{n_{2q}}}_{F_{gate}} \cdot \underbrace{\exp\left(-\frac{t_{total}}{T_2}\right)}_{F_{decoherence}}
$$

- $n_{1q}$, $n_{2q}$: 단일/이중 큐비트 게이트 수
- $\epsilon_{1q}$, $\epsilon_{2q}$: 게이트별 에러율
- $t_{total}$: 총 회로 실행 시간

---

## 2. 시뮬레이션 벤치마크 확장

### 2.1 실험 매트릭스

| 회로 유형 | 큐비트 수 | 게이트 수 | 반복 횟수 | 노이즈 모델 |
|----------|----------|----------|----------|------------|
| Bell State | 2 | 2 | 100 | Aer Noisy (mock) |
| GHZ State | 3, 5 | N-1 | 100 | Aer Noisy (mock) |
| QAOA (MaxCut) | 4 | 20 | 50 | Aer Noisy (mock) |
| VQE (H2) | 4 | 30 | 50 | Aer Noisy (mock) |

### 2.2 측정 지표

| 지표 | 정의 | 단위 |
|------|------|------|
| **Baseline Fidelity** | 원본 회로의 측정 충실도 | 0-1 |
| **QNS Fidelity** | LiveRewirer 적용 후 충실도 | 0-1 |
| **개선율** | (QNS - Baseline) / Baseline x 100 | % |
| **Rewiring 시간** | LiveRewirer 최적화 소요 시간 | ms |

### 2.3 예상 결과 테이블 형식

| Circuit | Qubits | Baseline Fidelity | QNS Fidelity | Improvement (%) | Rewire Time (ms) |
|---------|--------|-------------------|--------------|-----------------|------------------|
| Bell | 2 | 0.493 | 0.512 | +3.9% | 12 |
| GHZ-3 | 3 | 0.486 | 0.508 | +4.5% | 18 |
| GHZ-5 | 5 | TBD | TBD | TBD | TBD |
| QAOA | 4 | TBD | TBD | TBD | TBD |
| VQE | 4 | TBD | TBD | TBD | TBD |

---

## 3. 코드 구현 작업

| 파일 | 작업 내용 |
|------|----------|
| `qns_bench/src/aer_benchmark.rs` | Aer 백엔드 벤치마크 스크립트 작성 |
| `qns_bench/src/csv_export.rs` | 결과 CSV 내보내기 기능 |
| `docs/QNS_Benchmark_Results.md` | 벤치마크 결과 문서화 |

---

## 4. 재현성 정보

| 항목 | 값 |
|------|-----|
| Rust 버전 | 1.75.0 |
| Python 버전 | 3.11 |
| Qiskit 버전 | 1.0+ |
| 테스트 환경 | AMD Ryzen 9 / 16GB RAM |
| 난수 시드 | 42 (고정) |

---

## 5. 완료 기준 (Definition of Done)

- [x] LiveRewirer 목표 함수 수식 문서화 ✅ `QNS_Mathematical_Formalization.md`
- [x] Aer Noisy 벤치마크 테이블 완성 (최소 5개 회로) ✅ 5개 회로 완료
- [x] docs/ 폴더에 벤치마크 결과 문서 추가 ✅ `QNS_Benchmark_Results.md`
- [x] README.md 업데이트 ✅ 벤치마크 섹션 및 문서 링크 추가

---

**예상 소요 시간:** 1일

---

## 6. Gantree 기반 단계별 실행 계획서

> PPR/Gantree 방식에 따른 Top-Down BFS 설계. 원자화 노드까지 분해하여 AI 협업 작업에 최적화.

### 6.1 전체 구조 트리

```gantree
QNS_ArXiv_Upgrade // arXiv 논문 보강 프로젝트 (진행중)
    L1_MathFormalization // 수학적 형식화 보강 (설계중)
        LiveRewirer_ObjectiveFunc // 목표 함수 정의 (설계중)
        Fidelity_EstimationModel // 충실도 추정 모델 (설계중)
    L2_SimulationBenchmark // 시뮬레이션 벤치마크 (설계중)
        BenchmarkInfra // 벤치마크 인프라 구축 (설계중)
        CircuitExecution // 회로별 실험 수행 (설계중)
        ResultAnalysis // 결과 분석 및 문서화 (설계중)
    L3_CodeImplementation // 코드 구현 (설계중)
        AerBenchmarkModule // Aer 벤치마크 모듈 (설계중)
        CsvExportModule // CSV 내보내기 모듈 (설계중)
    L4_Documentation // 문서화 작업 (설계중)
        BenchmarkResultDoc // 벤치마크 결과 문서 (설계중)
        ReadmeUpdate // README 업데이트 (설계중)
```

---

### 6.2 L1: 수학적 형식화 보강 (분해)

```gantree
L1_MathFormalization // 수학적 형식화 보강 (설계중)
    LiveRewirer_ObjectiveFunc // 목표 함수 정의 (설계중)
        DefineVariantSet // V(C) 변종 집합 정의 (원자)
        DefineNoiseVector // n=(T1,T2,ε_g) 벡터 정의 (원자)
        FormulateArgMax // argmax 최적화 형식 작성 (원자)
        ValidateMathNotation // LaTeX 수식 검증 (원자)
    Fidelity_EstimationModel // 충실도 추정 모델 (설계중)
        DefineGateErrorProduct // 게이트 에러 곱 정의 (원자)
        DefineDecoherenceExp // T2 지수 감쇠 정의 (원자)
        IntegrateFidelityFormula // 통합 수식 작성 (원자)
```

**PPR 구현 예시:**

```python
def AI_make_objective_function():
    """LiveRewirer 최적화 목표 함수 LaTeX 정의"""
    return r"OptimalCircuit = \arg\max_{C' \in \mathcal{V}(C)} \hat{F}(C', \mathbf{n})"

def AI_make_fidelity_model():
    """충실도 추정 함수 LaTeX 정의"""
    return r"\hat{F}(C, \mathbf{n}) = \prod_{g \in C} (1 - \epsilon_g) \cdot \exp(-t_{total}/T_2)"
```

---

### 6.3 L2: 시뮬레이션 벤치마크 (분해)

```gantree
L2_SimulationBenchmark // 시뮬레이션 벤치마크 (설계중)
    BenchmarkInfra // 인프라 구축 (설계중)
        SetupAerNoisy // Aer Noisy 백엔드 설정 (원자)
        ConfigureRandomSeed // 난수 시드 42 고정 (원자)
        CreateMeasurementLoop // 100회 반복 측정 루프 (원자)
    CircuitExecution // 회로별 실험 수행 (설계중)
        BellState_Bench // Bell State 2큐비트 (원자)
        GHZ3_Bench // GHZ 3큐비트 (원자)
        GHZ5_Bench // GHZ 5큐비트 (원자)
        QAOA_Bench // QAOA MaxCut 4큐비트 (원자)
        VQE_Bench // VQE H2 4큐비트 (원자)
    ResultAnalysis // 결과 분석 (설계중)
        CalcBaselineFidelity // 기준 충실도 계산 (원자)
        CalcQNSFidelity // QNS 적용 충실도 계산 (원자)
        CalcImprovementRate // 개선율 % 계산 (원자)
        MeasureRewiringTime // Rewiring 시간 측정 (원자)
```

**PPR 구현 예시:**

```python
def AI_make_benchmark_suite():
    """벤치마크 회로 목록 생성"""
    return [
        {"name": "Bell", "qubits": 2, "gates": 2, "shots": 100},
        {"name": "GHZ-3", "qubits": 3, "gates": 2, "shots": 100},
        {"name": "GHZ-5", "qubits": 5, "gates": 4, "shots": 100},
        {"name": "QAOA", "qubits": 4, "gates": 20, "shots": 50},
        {"name": "VQE", "qubits": 4, "gates": 30, "shots": 50},
    ]
```

---

### 6.4 L3: 코드 구현 (분해)

```gantree
L3_CodeImplementation // 코드 구현 (설계중)
    AerBenchmarkModule // qns_bench/src/aer_benchmark.rs (설계중)
        CreateBenchStruct // 벤치마크 구조체 정의 (원자)
        ImplNoiseModel // 노이즈 모델 구현 (원자)
        ImplCircuitRunner // 회로 실행기 구현 (원자)
        ImplFidelityCalc // 충실도 계산기 구현 (원자)
    CsvExportModule // qns_bench/src/csv_export.rs (설계중)
        DefineResultStruct // 결과 구조체 정의 (원자)
        ImplCsvWriter // CSV 작성기 구현 (원자)
        ImplHeaderGen // 헤더 생성 구현 (원자)
```

---

### 6.5 L4: 문서화 (분해)

```gantree
L4_Documentation // 문서화 작업 (설계중)
    BenchmarkResultDoc // docs/QNS_Benchmark_Results.md (설계중)
        WriteIntroduction // 서론 작성 (원자)
        CreateResultTable // 결과 테이블 생성 (원자)
        AddVisualization // 그래프/차트 추가 (원자)
        WriteConclusion // 결론 작성 (원자)
    ReadmeUpdate // README.md 업데이트 (설계중)
        AddBenchmarkSection // 벤치마크 섹션 추가 (원자)
        UpdateQuickStart // Quick Start 갱신 (원자)
```

---

### 6.6 실행 순서 (BFS 순서)

| 단계 | 작업 | 선행 조건 | 예상 시간 |
|------|------|----------|----------|
| 1 | L1_MathFormalization | 없음 | 1시간 |
| 2 | L3_AerBenchmarkModule | 없음 | 2시간 |
| 3 | L3_CsvExportModule | L3_AerBenchmark | 30분 |
| 4 | L2_BenchmarkInfra | L3 완료 | 1시간 |
| 5 | L2_CircuitExecution | L2_Infra | 2시간 |
| 6 | L2_ResultAnalysis | L2_Execution | 30분 |
| 7 | L4_BenchmarkResultDoc | L2 완료 | 1시간 |
| 8 | L4_ReadmeUpdate | L4_ResultDoc | 30분 |

---

### 6.7 검증 체크리스트

✅ **수학적 형식화**

- [ ] LaTeX 수식 렌더링 정상 확인
- [ ] 변수 정의 명확성 검토

✅ **벤치마크 실행**

- [ ] 모든 5개 회로 테스트 통과
- [ ] 난수 시드 42로 재현성 확인
- [ ] 결과 CSV 파일 정상 생성

✅ **문서화**

- [ ] docs/QNS_Benchmark_Results.md 완성
- [ ] README.md 벤치마크 섹션 추가
