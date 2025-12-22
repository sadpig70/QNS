# QNS Phase 3 - Sprint 기반 구현 계획서

> **기준 문서**: QNS_Phase3_Upgrade_Gantree.md
> **목표**: 40% → 70% 완성도 달성
> **총 예상 기간**: 7주 (Sprint 1-7)
> **작성일**: 2025-12-10

---

## 개요

이 문서는 Phase 3 저널 게재를 위한 구체적인 구현 계획입니다.
각 Sprint는 1주 단위이며, 매 Sprint 종료 시 검증 가능한 결과물을 생성합니다.

---

## Sprint 1: Scoring 모듈 완성 (Week 1)

> **목표**: estimate_fidelity_with_scheduling() 실제 구현
> **담당 파일**: `crates/qns_rewire/src/scoring.rs`

### Day 1-2: decay_estimation 구현

```
Task 1.1: decay_estimation // T1/T2 기반 감쇠 계산 (원자)
    파일: scoring.rs
    입력: makespan (f64), t1 (f64), t2 (f64)
    출력: decay_probability (f64)
    공식: 1 - exp(-makespan / T1) * exp(-makespan / T2)

    테스트 케이스:
    - [ ] makespan=0 → decay=0
    - [ ] makespan=T1 → decay≈0.632
    - [ ] T2 > 2*T1 → Warning 로그 + T2=2*T1로 클램핑 (물리적 제약)
    - [ ] T1 또는 T2 <= 0 → Error 반환

    엣지 케이스 처리:
    - T2 > 2*T1: 물리적으로 불가능, 클램핑 후 경고
    - makespan < 0: 에러 반환
    - T1/T2 = inf: decay = 0 (이상적 큐빗)

    예상 시간: 2h
    의존성: 없음
```

### Day 2-3: gate_error_sum 구현

```
Task 1.2: gate_error_sum // 게이트 에러 누적 (원자)
    파일: scoring.rs
    입력: circuit (&CircuitGenome), noise (&NoiseVector)
    출력: total_error (f64)
    로직:
        - 1-qubit 게이트: noise.gate_error_1q * count_1q
        - 2-qubit 게이트: noise.gate_error_2q * count_2q
        - 측정: noise.readout_error * measure_count

    테스트 케이스:
    - [ ] 빈 회로 → error=0
    - [ ] H(0) 1개 → error=gate_error_1q
    - [ ] CNOT(0,1) 1개 → error=gate_error_2q
    - [ ] 혼합 회로 → 정확한 누적

    예상 시간: 2h
    의존성: 없음
```

### Day 3-4: critical_path 계산

```
Task 1.3: critical_path // Makespan 계산 (원자)
    파일: scoring.rs
    입력: circuit (&CircuitGenome), config (&ScoreConfig)
    출력: makespan_ns (f64)
    로직:
        - 각 큐빗별 게이트 실행 시간 추적
        - 2-qubit 게이트는 두 큐빗 중 늦은 시점에서 시작
        - 최대 종료 시간 = makespan

    테스트 케이스:
    - [ ] 직렬 회로: 모든 게이트 시간 합
    - [ ] 병렬 회로: max(qubit별 시간)
    - [ ] 혼합 회로: 정확한 임계 경로

    예상 시간: 3h
    의존성: 없음
```

### Day 4-5: 통합 및 테스트

```
Task 1.4: estimate_fidelity_with_scheduling 통합 (원자)
    파일: scoring.rs
    로직:
        makespan = critical_path(circuit, config)
        decay = decay_estimation(makespan, noise.t1, noise.t2)
        gate_err = gate_error_sum(circuit, noise)
        fidelity = (1 - decay) * (1 - gate_err)
        return fidelity.clamp(0.0, 1.0)

    테스트 케이스:
    - [ ] 이상적 노이즈 (T1=∞, error=0) → fidelity=1.0
    - [ ] 실제적 노이즈 → 0 < fidelity < 1
    - [ ] 극단적 노이즈 → fidelity≈0

    예상 시간: 2h
    의존성: Task 1.1, 1.2, 1.3
```

### Sprint 1 완료 기준

- [ ] `cargo test -p qns_rewire scoring` 전체 통과
- [ ] 벤치마크: 10,000회 스코어링 < 100ms
- [ ] 문서: 각 함수에 doc comment 작성

---

## Sprint 2: LiveRewirer 핵심 로직 (Week 2)

> **목표**: optimize()가 실제로 최적 변종을 선택
> **담당 파일**: `crates/qns_rewire/src/live_rewirer/mod.rs`

### Day 1-2: score_all_variants 구현

```
Task 2.1: score_all_variants // 모든 변종 스코어링 (원자)
    파일: live_rewirer/mod.rs
    입력: variants (Vec<CircuitGenome>), noise (&NoiseVector)
    출력: Vec<(CircuitGenome, f64)>
    로직:
        variants.iter()
            .map(|v| (v.clone(), estimate_fidelity_with_scheduling(v, noise, config)))
            .collect()

    예상 시간: 1h
    의존성: Sprint 1 완료
```

### Day 2-3: find_best_variant 구현

```
Task 2.2: find_best_variant // 최적 변종 선택 (원자)
    파일: live_rewirer/mod.rs
    입력: scored_variants (Vec<(CircuitGenome, f64)>)
    출력: (best_circuit, best_score, improvement)
    로직:
        scored_variants.iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())

    예상 시간: 1h
    의존성: Task 2.1
```

### Day 3-4: parallel_evaluation (Rayon)

```
Task 2.3: parallel_evaluation // 병렬 스코어링 (원자)
    파일: live_rewirer/mod.rs
    의존성 추가: rayon = "1.8"
    입력: variants (Vec<CircuitGenome>), noise (&NoiseVector)
    출력: Vec<(CircuitGenome, f64)>
    로직:
        use rayon::prelude::*;
        variants.par_iter()
            .map(|v| (v.clone(), estimate_fidelity(...)))
            .collect()

    예상 시간: 2h
    의존성: Task 2.1
```

### Day 4-5: optimize() 완성

```
Task 2.4: optimize() 전체 통합 (원자)
    파일: live_rewirer/mod.rs
    로직:
        1. variants = gate_reorder.generate_reorderings(circuit)
        2. scored = parallel_evaluation(variants, noise)
        3. (best, score, improvement) = find_best_variant(scored)
        4. return OptimizationResult { ... }

    테스트 케이스:
    - [ ] 빈 회로 → 원본 반환
    - [ ] 단일 게이트 → 원본 반환
    - [ ] 교환 가능 회로 → improved=true (노이즈에 따라)
    - [ ] 교환 불가 회로 → improved=false

    예상 시간: 3h
    의존성: Task 2.1, 2.2, 2.3
```

### Sprint 2 완료 기준

- [ ] `cargo test -p qns_rewire live_rewirer` 전체 통과
- [ ] LiveRewirer.optimize()가 실제로 다른 회로 반환 (교환 가능 시)
- [ ] 벤치마크: 20 게이트 회로 최적화 < 50ms

---

## Sprint 3: 노이즈 시뮬레이터 강화 (Week 3)

> **목표**: Kraus 연산자 기반 노이즈 시뮬레이션
> **담당 파일**: `crates/qns_simulator/src/noisy.rs`, `noise.rs`

### Day 1-2: Kraus 연산자 구현

```
Task 3.1: kraus_operators // Kraus 연산자 적용 (원자)
    파일: noisy.rs
    입력: state_vector, kraus_ops (Vec<Matrix>)
    출력: new_state_vector (확률적)
    로직:
        p_i = trace(K_i * rho * K_i†)
        선택된 i에 대해: rho' = K_i * rho * K_i† / p_i

    테스트 케이스:
    - [ ] Identity Kraus → 상태 불변
    - [ ] Full depolarizing → 완전 혼합 상태

    예상 시간: 4h
    의존성: 없음
```

### Day 2-3: amplitude_damping 채널

```
Task 3.2: amplitude_damping // 진폭 감쇠 (원자)
    파일: noise.rs
    입력: gamma (감쇠율), state_vector
    출력: damped_state
    Kraus 연산자:
        K0 = [[1, 0], [0, sqrt(1-gamma)]]
        K1 = [[0, sqrt(gamma)], [0, 0]]

    테스트 케이스:
    - [ ] gamma=0 → 상태 불변
    - [ ] gamma=1 → |1⟩ → |0⟩

    예상 시간: 2h
    의존성: Task 3.1
```

### Day 3-4: phase_damping 채널

```
Task 3.3: phase_damping // 위상 감쇠 (원자)
    파일: noise.rs
    입력: gamma (감쇠율), state_vector
    출력: damped_state
    Kraus 연산자:
        K0 = [[1, 0], [0, sqrt(1-gamma)]]
        K1 = [[0, 0], [0, sqrt(gamma)]]

    예상 시간: 2h
    의존성: Task 3.1
```

### Day 4-5: NoisySimulator 통합

```
Task 3.4: NoisySimulator 업데이트 (원자)
    파일: noisy.rs
    로직:
        각 게이트 적용 후:
        1. T1 기반 amplitude_damping 적용
        2. T2 기반 phase_damping 적용
        3. gate_error 확률로 depolarizing 적용

    테스트 케이스:
    - [ ] ideal 모드 → StateVectorSimulator와 동일
    - [ ] noisy 모드 → fidelity < 1.0

    예상 시간: 3h
    의존성: Task 3.2, 3.3
```

### Sprint 3 완료 기준

- [ ] `cargo test -p qns_simulator noisy` 전체 통과
- [ ] Before/After QNS 최적화 fidelity 비교 가능
- [ ] 노이즈 강도에 따른 fidelity 감소 검증

---

## Sprint 4: 벤치마크 인프라 (Week 4)

> **목표**: Qiskit 비교 벤치마크 실행 가능
> **담당 파일**: `scripts/`, `benchmarks/`

### Day 1-2: QASMBench 임포트

```
Task 4.1: qasmbench_import // QASMBench 회로 임포트 (원자)
    파일: benchmarks/qasmbench/
    작업:
        1. QASMBench GitHub에서 회로 다운로드
        2. 파일명 표준화 (algo_nX.qasm)
        3. 각 회로 파싱 테스트

    목표 회로 (최소 20개):
        - qft_n4, qft_n6, qft_n8
        - grover_n4, grover_n6
        - qaoa_n4, qaoa_n6
        - vqe_n4, vqe_n6
        - ...

    예상 시간: 3h
    의존성: 없음
```

### Day 2-3: Qiskit 비교 스크립트

```
Task 4.2: qiskit_integration // Qiskit 연동 스크립트 (원자)
    파일: scripts/compare_qiskit.py
    작업:
        1. QASM 파일 로드
        2. Qiskit transpile() 실행
        3. QNS optimize() 실행 (subprocess or PyO3)
        4. 결과 비교 (depth, gate_count, fidelity)

    출력 형식:
        {
            "circuit": "qft_n4",
            "qiskit": {"depth": 12, "gates": 45, "time_ms": 23},
            "qns": {"depth": 10, "gates": 42, "time_ms": 15},
            "improvement": {"depth": "16.7%", "gates": "6.7%"}
        }

    예상 시간: 4h
    의존성: Task 4.1
```

### Day 3-4: 벤치마크 자동화

```
Task 4.3: benchmark_runner // 벤치마크 실행기 (원자)
    파일: scripts/run_benchmark.py
    작업:
        1. benchmarks/ 폴더 순회
        2. 각 회로에 대해 compare_qiskit.py 실행
        3. 결과 집계 (JSON, CSV)
        4. 통계 계산 (mean, std, p-value)

    예상 시간: 3h
    의존성: Task 4.2
```

### Day 4-5: 리포트 생성

```
Task 4.4: report_generation // 리포트 생성 (원자)
    파일: scripts/generate_report.py
    출력:
        1. results/benchmark_results.json
        2. results/benchmark_results.csv
        3. results/figures/comparison_plot.png
        4. results/tables/latex_table.tex

    예상 시간: 3h
    의존성: Task 4.3
```

### Sprint 4 완료 기준

- [ ] `python scripts/run_benchmark.py` 정상 실행
- [ ] 20+ 회로 벤치마크 결과 생성
- [ ] Qiskit 대비 결과 테이블 출력

---

## Sprint 5: 통계적 검증 (Week 5)

> **목표**: p < 0.05 유의성 검증
> **담당 파일**: `scripts/`, 테스트 코드

### Day 1-2: 다중 실행 프레임워크

```
Task 5.1: multiple_runs // 다중 실행 (원자)
    파일: scripts/statistical_validation.py
    작업:
        1. 각 회로에 대해 N=100 반복 실행
        2. 노이즈 시드 무작위화
        3. fidelity 결과 수집

    예상 시간: 3h
    의존성: Sprint 4 완료
```

### Day 2-3: 신뢰 구간 계산

```
Task 5.2: confidence_intervals // 신뢰 구간 (원자)
    파일: scripts/statistical_validation.py
    작업:
        1. 95% 신뢰 구간 계산
        2. Bootstrap 방법 적용
        3. 결과 시각화

    예상 시간: 2h
    의존성: Task 5.1
```

### Day 3-4: 가설 검정

```
Task 5.3: hypothesis_testing // 가설 검정 (원자)
    파일: scripts/statistical_validation.py
    작업:
        H0: QNS fidelity <= Baseline fidelity
        H1: QNS fidelity > Baseline fidelity

        1. Paired t-test 수행
        2. Wilcoxon signed-rank test (비모수)
        3. Effect size (Cohen's d) 계산
        4. p-value 보고

    예상 시간: 3h
    의존성: Task 5.1
```

### Day 4-5: Ablation Study

```
Task 5.4: ablation_study // Ablation 분석 (원자)
    파일: scripts/ablation_study.py
    실험:
        1. Reorder Only: 재정렬만, 스코어링 없음
        2. Scoring Only: 스코어링만, 재정렬 없음
        3. Full Pipeline: 전체 파이프라인
        4. 각 조합의 fidelity 비교

    예상 시간: 3h
    의존성: Task 5.1
```

### Sprint 5 완료 기준

- [ ] p-value < 0.05 달성 (주요 회로군)
- [ ] 95% 신뢰 구간 그래프 생성
- [ ] Ablation 결과 테이블 완성

---

## Sprint 6: Python 바인딩 완성 (Week 6)

> **목표**: Qiskit 직접 연동 가능
> **담당 파일**: `crates/qns_python/`

### Day 1-2: 회로 변환

```
Task 6.1: qiskit_circuit_convert // Qiskit 회로 변환 (원자)
    파일: qns_python/src/convert.rs
    작업:
        1. QuantumCircuit → CircuitGenome 변환
        2. CircuitGenome → QuantumCircuit 변환
        3. 게이트 매핑 테이블 작성

    지원 게이트: H, X, Y, Z, S, T, Rx, Ry, Rz, CX, CZ, SWAP

    예상 시간: 4h
    의존성: 없음
```

### Day 2-3: 노이즈 모델 변환

```
Task 6.2: noise_model_convert // 노이즈 모델 변환 (원자)
    파일: qns_python/src/convert.rs
    작업:
        1. IBM Backend calibration → NoiseVector
        2. Qiskit NoiseModel → QNS NoiseModel

    예상 시간: 3h
    의존성: 없음
```

### Day 3-4: PyO3 래퍼 완성

```
Task 6.3: pyo3_wrapper // PyO3 래퍼 (원자)
    파일: qns_python/src/lib.rs
    구현:
        #[pyclass]
        struct QnsOptimizer { ... }

        #[pymethods]
        impl QnsOptimizer {
            fn from_qiskit_circuit(&mut self, qc: PyObject) -> PyResult<()>
            fn optimize(&self, noise: &NoiseModel) -> PyResult<OptimizationResult>
            fn to_qiskit_circuit(&self) -> PyResult<PyObject>
        }

    예상 시간: 4h
    의존성: Task 6.1, 6.2
```

### Day 4-5: 테스트 및 문서화

```
Task 6.4: python_tests // Python 테스트 (원자)
    파일: qns_python/tests/test_qiskit_integration.py
    테스트:
        1. 회로 왕복 변환 (Qiskit → QNS → Qiskit)
        2. 최적화 후 fidelity 비교
        3. IBM fake backend 연동

    예상 시간: 3h
    의존성: Task 6.3
```

### Sprint 6 완료 기준

- [ ] `pip install .` 성공 (maturin)
- [ ] `from qns import QnsOptimizer` 동작
- [ ] Qiskit 회로 최적화 예제 실행

---

## Sprint 7: 논문 준비 및 마무리 (Week 7)

> **목표**: 논문 초안 완성, 재현성 패키지 준비
> **담당**: docs/, scripts/

### Day 1-2: 그림 및 테이블 준비

```
Task 7.1: figures_and_tables // 그림/테이블 (원자)
    작업:
        1. 아키텍처 다이어그램 (draw.io)
        2. 알고리즘 플로우차트
        3. 벤치마크 비교 그래프 (matplotlib)
        4. 결과 테이블 (LaTeX)

    예상 시간: 4h
    의존성: Sprint 5 완료
```

### Day 2-3: 원고 초안

```
Task 7.2: manuscript_draft // 원고 초안 (원자)
    섹션:
        1. Abstract (200 words)
        2. Introduction (동기, 기여)
        3. Related Work (기존 연구)
        4. Methodology (알고리즘)
        5. Experiments (결과)
        6. Discussion (한계)
        7. Conclusion

    예상 시간: 8h (2일)
    의존성: Task 7.1
```

### Day 4: 재현성 패키지

```
Task 7.3: reproducibility_package // 재현성 (원자)
    작업:
        1. GitHub 릴리스 태그
        2. Docker 이미지 (Dockerfile)
        3. README 업데이트
        4. 벤치마크 실행 스크립트

    예상 시간: 3h
    의존성: 없음
```

### Day 5: 최종 검증

```
Task 7.4: final_validation // 최종 검증 (원자)
    작업:
        1. 전체 테스트 스위트 실행
        2. 벤치마크 재실행 (결과 일관성)
        3. Docker 이미지 테스트
        4. 코드 리뷰 체크리스트

    예상 시간: 4h
    의존성: 모든 이전 Task
```

### Sprint 7 완료 기준

- [ ] 논문 초안 완성 (PDF)
- [ ] Docker 이미지 빌드 및 테스트
- [ ] GitHub 릴리스 준비 완료

---

## 전체 일정 요약

```
Week 1: [████████████] Sprint 1 - Scoring 모듈
Week 2: [████████████] Sprint 2 - LiveRewirer
Week 3: [████████████] Sprint 3 - 노이즈 시뮬레이터
Week 4: [████████████] Sprint 4 - 벤치마크 인프라
Week 5: [████████████] Sprint 5 - 통계적 검증
Week 6: [████████████] Sprint 6 - Python 바인딩
Week 7: [████████████] Sprint 7 - 논문 준비
```

---

## 리스크 버퍼

| Sprint | 버퍼 | 용도 |
|--------|------|------|
| Sprint 2 | +2일 | Rayon 통합 이슈 |
| Sprint 3 | +2일 | Kraus 연산자 수치 안정성 이슈 |
| Sprint 4 | +2일 | Qiskit API 변경 대응 |
| Sprint 5 | +1일 | 통계적 유의성 미달 시 샘플 증가 |
| Sprint 6 | +1일 | PyO3 메모리 관리 이슈 |

---

## 일일 체크리스트 템플릿

```markdown
## Daily Standup - Sprint X, Day Y

### 어제 완료
- [ ] Task X.Y 완료

### 오늘 계획
- [ ] Task X.Z 시작

### 블로커
- (없음 / 설명)

### 메모
-
```

---

## 품질 게이트

### 각 Sprint 종료 시

1. **코드 품질**
   - [ ] `cargo clippy --all-targets` 경고 0
   - [ ] `cargo fmt --check` 통과
   - [ ] 새 함수에 doc comment 작성

2. **테스트**
   - [ ] 새 코드 테스트 커버리지 80%+
   - [ ] 기존 테스트 회귀 없음
   - [ ] 통합 테스트 추가

3. **문서**
   - [ ] CHANGELOG 업데이트
   - [ ] README 필요 시 업데이트

---

---

## Sprint 의존성 그래프

```text
Sprint 1 (Scoring)
    ↓
Sprint 2 (LiveRewirer) ← Sprint 1 필수
    ↓
Sprint 3 (Noisy Sim) ← Sprint 1, 2 권장
    ↓
Sprint 4 (Benchmark) ← Sprint 2, 3 필수
    ↓
Sprint 5 (Statistics) ← Sprint 4 필수
    ↓
Sprint 6 (Python) ← Sprint 2 필수 (병렬 가능: Sprint 3-5)
    ↓
Sprint 7 (Paper) ← Sprint 5, 6 필수
```

---

## CI/CD 통합 계획

각 Sprint PR 병합 전 자동 검증:

```yaml
# .github/workflows/sprint_validation.yml
name: Sprint Validation

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build
        run: cargo build --all-targets

      - name: Clippy
        run: cargo clippy --all-targets -- -D warnings

      - name: Test
        run: cargo test --all

      - name: Format Check
        run: cargo fmt --check

      - name: Coverage (Sprint 완료 시)
        run: cargo tarpaulin --out Xml
        if: contains(github.event.head_commit.message, '[sprint-complete]')
```

---

*이 계획서는 QNS Phase 3 업그레이드의 구체적인 실행 로드맵입니다.*
*각 Task는 명확한 입출력과 테스트 케이스를 포함합니다.*
