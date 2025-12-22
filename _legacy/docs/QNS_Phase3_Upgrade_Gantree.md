# QNS Phase 3 Journal Upgrade - Gantree Design

> **Target Journals**: Quantum Science and Technology (IF 5.6), npj Quantum Information (IF 6.3), Quantum (IF 6.4)
> **Version**: 1.0
> **Created**: 2025-12-10
> **Goal**: 프로젝트 완성도 40% → 70% 달성

---

## 1. Master Gantree

```
QNS_Phase3_Upgrade // Phase 3 저널 게재를 위한 업그레이드 (설계중)
    L1_AlgorithmCompletion // 핵심 알고리즘 완성 (설계중)
    L1_BenchmarkInfra // 벤치마크 인프라 구축 (설계중)
    L1_ExperimentalValidation // 실험적 검증 (설계중)
    L1_PublicationPrep // 논문 준비 (설계중)
```

---

## 2. L1_AlgorithmCompletion (분해)

> **목표**: 스켈레톤 상태인 핵심 모듈 완전 구현
> **예상 시간**: 4-6주
> **우선순위**: Critical

```
L1_AlgorithmCompletion // 핵심 알고리즘 완성 (설계중)
    L2_LiveRewirer // 실시간 최적화 엔진 완성 (설계중)
        L3_OptimizeCore // optimize() 핵심 로직 (설계중)
            find_best_variant // 최적 변종 탐색 (설계중)
                score_all_variants // 모든 변종 스코어링 (원자)
                parallel_evaluation // Rayon 병렬 평가 (원자)
                select_top_k // 상위 k개 선택 (원자)
            adaptive_search // 적응형 탐색 전략 (설계중)
                beam_search_phase // Beam Search 단계 (완료)
                bfs_refinement // BFS 정제 단계 (원자)
                early_termination // 조기 종료 조건 (원자)
        L3_HardwareAware // 하드웨어 인식 최적화 (설계중)
            topology_constraint // 토폴로지 제약 적용 (원자)
            coupling_map_check // 커플링 맵 검증 (원자)
            swap_insertion // SWAP 게이트 삽입 (원자)
        L3_FidelityModel // 피델리티 추정 모델 (설계중)
            decay_estimation // T1/T2 감쇠 추정 (원자)
            gate_error_sum // 게이트 에러 누적 (원자)
            crosstalk_penalty // 크로스톡 패널티 (원자)
    L2_Scoring // 스코어링 모듈 완성 (설계중)
        L3_MakespanCalc // Makespan 계산 (설계중)
            critical_path // 임계 경로 분석 (원자)
            parallel_gate_detection // 병렬 게이트 감지 (원자)
            total_time_estimation // 총 실행 시간 추정 (원자)
        L3_FidelityEstimation // 피델리티 추정 (설계중)
            idle_decay // 유휴 시간 감쇠 (원자)
            gate_fidelity_product // 게이트 피델리티 곱 (원자)
            measurement_error // 측정 에러 반영 (원자)
    L2_SimulatorIntegration // 시뮬레이터 통합 (설계중)
        L3_NoisySimulator // 노이즈 시뮬레이터 (진행중)
            kraus_operators // Kraus 연산자 적용 (원자)
            amplitude_damping // 진폭 감쇠 채널 (원자)
            phase_damping // 위상 감쇠 채널 (원자)
        L3_ValidationSim // 검증용 시뮬레이터 (설계중)
            ideal_vs_noisy // 이상 vs 노이즈 비교 (원자)
            fidelity_measurement // 피델리티 측정 (완료)
            statistical_sampling // 통계적 샘플링 (원자)
```

---

## 3. L1_BenchmarkInfra (분해)

> **목표**: 재현 가능한 벤치마크 시스템 구축
> **예상 시간**: 2-3주
> **우선순위**: High

```
L1_BenchmarkInfra // 벤치마크 인프라 구축 (설계중)
    L2_CircuitSuite // 벤치마크 회로 스위트 (설계중)
        L3_StandardBench // 표준 벤치마크 (설계중)
            qasmbench_import // QASMBench 회로 임포트 (원자)
            revlib_import // RevLib 회로 임포트 (원자)
            custom_circuits // 커스텀 테스트 회로 (원자)
        L3_ScalabilityTest // 확장성 테스트 (설계중)
            qubit_scaling // 큐빗 수 스케일링 (2-20) (원자)
            depth_scaling // 회로 깊이 스케일링 (원자)
            gate_count_scaling // 게이트 수 스케일링 (원자)
    L2_CompetitorComparison // 경쟁 도구 비교 (설계중)
        L3_QiskitBaseline // Qiskit Transpiler 비교 (설계중)
            qiskit_integration // Qiskit 연동 스크립트 (원자)
            transpile_benchmark // Transpile 벤치마크 (원자)
            fidelity_comparison // 피델리티 비교 (원자)
        L3_CirqBaseline // Cirq 비교 (설계중)
            cirq_integration // Cirq 연동 스크립트 (원자)
            optimization_benchmark // 최적화 벤치마크 (원자)
        L3_TketBaseline // t|ket> 비교 (보류)
            tket_integration // t|ket> 연동 (원자)
    L2_MetricsCollection // 메트릭 수집 (설계중)
        L3_PerformanceMetrics // 성능 메트릭 (설계중)
            latency_measurement // 지연 시간 측정 (원자)
            throughput_measurement // 처리량 측정 (원자)
            memory_usage // 메모리 사용량 (원자)
        L3_QualityMetrics // 품질 메트릭 (설계중)
            fidelity_improvement // 피델리티 향상률 (원자)
            gate_reduction // 게이트 감소율 (원자)
            depth_reduction // 깊이 감소율 (원자)
    L2_ReportGeneration // 리포트 생성 (설계중)
        json_export // JSON 결과 내보내기 (원자)
        csv_export // CSV 테이블 내보내기 (원자)
        plot_generation // 그래프 생성 (matplotlib) (원자)
        latex_table // LaTeX 테이블 생성 (원자)
```

---

## 4. L1_ExperimentalValidation (분해)

> **목표**: 시뮬레이터 기반 실험적 검증
> **예상 시간**: 3-4주
> **우선순위**: High

```
L1_ExperimentalValidation // 실험적 검증 (설계중)
    L2_SimulatorExperiments // 시뮬레이터 실험 (설계중)
        L3_NoiseModelValidation // 노이즈 모델 검증 (설계중)
            ibm_noise_params // IBM 공개 노이즈 파라미터 적용 (원자)
            noise_model_accuracy // 노이즈 모델 정확도 검증 (원자)
            sensitivity_analysis // 민감도 분석 (원자)
        L3_OptimizationEffectiveness // 최적화 효과 검증 (설계중)
            before_after_comparison // Before/After 비교 (원자)
            noise_level_sweep // 노이즈 레벨 스윕 (원자)
            circuit_type_analysis // 회로 유형별 분석 (원자)
        L3_StatisticalSignificance // 통계적 유의성 (설계중)
            multiple_runs // 다중 실행 (N=100) (원자)
            confidence_intervals // 신뢰 구간 계산 (원자)
            hypothesis_testing // 가설 검정 (p-value) (원자)
    L2_AblationStudy // Ablation Study (설계중)
        L3_ComponentAnalysis // 컴포넌트 분석 (설계중)
            reorder_only // 재정렬만 적용 (원자)
            scoring_only // 스코어링만 적용 (원자)
            full_pipeline // 전체 파이프라인 (원자)
        L3_ParameterSensitivity // 파라미터 민감도 (설계중)
            beam_width_sweep // Beam 폭 스윕 (원자)
            max_variants_sweep // 최대 변종 수 스윕 (원자)
            lookahead_depth_sweep // Lookahead 깊이 스윕 (원자)
    L2_EdgeCaseAnalysis // 엣지 케이스 분석 (설계중)
        empty_circuit // 빈 회로 처리 (완료)
        single_gate // 단일 게이트 회로 (완료)
        no_commuting_gates // 교환 불가 게이트만 (원자)
        max_qubit_circuit // 최대 큐빗 회로 (원자)
        deep_circuit // 매우 깊은 회로 (원자)
```

---

## 5. L1_PublicationPrep (분해)

> **목표**: 논문 작성 및 제출 준비
> **예상 시간**: 2-3주
> **우선순위**: Medium (알고리즘 완성 후)

```
L1_PublicationPrep // 논문 준비 (설계중)
    L2_Manuscript // 원고 작성 (설계중)
        L3_Sections // 섹션별 작성 (설계중)
            abstract_draft // 초록 작성 (원자)
            introduction // 서론 (배경, 동기) (원자)
            related_work // 관련 연구 (원자)
            methodology // 방법론 (알고리즘 설명) (원자)
            experiments // 실험 결과 (원자)
            discussion // 논의 및 한계 (원자)
            conclusion // 결론 (원자)
        L3_Figures // 그림 준비 (설계중)
            architecture_diagram // 아키텍처 다이어그램 (원자)
            algorithm_flowchart // 알고리즘 플로우차트 (원자)
            benchmark_plots // 벤치마크 그래프 (원자)
            noise_model_illustration // 노이즈 모델 도식 (원자)
        L3_Tables // 테이블 준비 (설계중)
            benchmark_results // 벤치마크 결과표 (원자)
            comparison_table // 비교 테이블 (원자)
            ablation_results // Ablation 결과표 (원자)
    L2_Reproducibility // 재현성 패키지 (설계중)
        github_release // GitHub 릴리스 준비 (원자)
        docker_container // Docker 컨테이너 (원자)
        benchmark_scripts // 벤치마크 스크립트 공개 (원자)
        data_availability // 데이터 공개 (원자)
    L2_Supplementary // 보충 자료 (설계중)
        detailed_proofs // 상세 증명 (원자)
        additional_experiments // 추가 실험 (원자)
        implementation_details // 구현 세부사항 (원자)
```

---

## 6. 원자 노드 상세 정의

### 6.1 Critical Path (최우선 구현)

| 노드 | 파일 | 예상 시간 | 의존성 |
|------|------|----------|--------|
| `score_all_variants` | `scoring.rs` | 2h | None |
| `decay_estimation` | `scoring.rs` | 1h | None |
| `gate_error_sum` | `scoring.rs` | 1h | None |
| `find_best_variant` | `live_rewirer/mod.rs` | 3h | score_all_variants |
| `parallel_evaluation` | `live_rewirer/mod.rs` | 2h | find_best_variant |
| `kraus_operators` | `noisy.rs` | 3h | None |
| `before_after_comparison` | `benchmarks/` | 2h | All above |

### 6.2 PPR DSL 매핑

```python
# L2_LiveRewirer → PPR 변환
class LiveRewirer:
    def AI_make_optimize(self, circuit, noise):
        """Gantree: L2_LiveRewirer/L3_OptimizeCore"""
        variants = AI_generate_variants(circuit)
        scores = AI_score_all(variants, noise)
        return AI_select_best(variants, scores)

# L2_Scoring → PPR 변환
class Scorer:
    def AI_make_estimate_fidelity(self, circuit, noise):
        """Gantree: L2_Scoring/L3_FidelityEstimation"""
        makespan = AI_calculate_makespan(circuit)
        decay = AI_estimate_decay(makespan, noise)
        gate_errors = AI_sum_gate_errors(circuit, noise)
        return 1.0 - decay - gate_errors
```

---

## 7. 구현 우선순위 매트릭스

```
긴급도 ↑
    │
    │  [L2_LiveRewirer]     [L2_Scoring]
    │  - find_best_variant  - decay_estimation
    │  - parallel_eval      - gate_error_sum
    │
    │  [L3_NoiseModelValid] [L2_CircuitSuite]
    │  - ibm_noise_params   - qasmbench_import
    │
    │  [L2_CompetitorComp]  [L2_Manuscript]
    │  - qiskit_baseline    - methodology
    │
    └──────────────────────────────────────→ 중요도
```

---

## 8. 마일스톤

| 마일스톤 | 목표 완성도 | 예상 기간 | 검증 기준 |
|----------|------------|----------|----------|
| **M1: Algorithm Core** | 55% | 2주 | LiveRewirer.optimize() 실제 최적화 수행 |
| **M2: Benchmark Ready** | 65% | 1주 | Qiskit 대비 벤치마크 실행 가능 |
| **M3: Validation Complete** | 70% | 2주 | 통계적 유의성 검증 완료 |
| **M4: Paper Draft** | 75% | 2주 | 원고 초안 완성 |

---

## 9. 리스크 및 대응

| 리스크 | 확률 | 영향 | 대응 |
|--------|------|------|------|
| Qiskit 대비 성능 열위 | 중 | 높음 | 특정 회로 유형에 특화된 강점 강조 |
| 통계적 유의성 미달 | 낮 | 높음 | 샘플 수 증가, 효과 크기 분석 |
| 구현 복잡도 과소평가 | 중 | 중 | 점진적 구현, 테스트 주도 개발 |
| 노이즈 모델 부정확 | 중 | 중 | IBM 공개 데이터 기반 검증 |

---

## 10. 성공 기준

### Phase 3 저널 게재 요건 충족 체크리스트

- [ ] **알고리즘 완성**: LiveRewirer가 실제 최적화 수행
- [ ] **벤치마크 비교**: Qiskit Transpiler 대비 결과 제시
- [ ] **통계적 검증**: p < 0.05 유의성 확보
- [ ] **재현성**: GitHub 공개 + 스크립트 제공
- [ ] **명확한 기여**: "Noise-Adaptive Reordering"의 novelty 입증
- [ ] **한계 명시**: 어떤 경우에 효과가 없는지 명확히 기술

---

## 11. 의존성 그래프 (Dependency Graph)

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Critical Path                                 │
└─────────────────────────────────────────────────────────────────────┘

[decay_estimation] ──┐
[gate_error_sum] ────┼──▶ [score_all_variants] ──▶ [find_best_variant]
[critical_path] ─────┘                                      │
                                                            ▼
[kraus_operators] ──▶ [ideal_vs_noisy] ──▶ [before_after_comparison]
                                                            │
                                                            ▼
[qiskit_integration] ──▶ [transpile_benchmark] ──▶ [fidelity_comparison]
                                                            │
                                                            ▼
[multiple_runs] ──▶ [confidence_intervals] ──▶ [hypothesis_testing]
                                                            │
                                                            ▼
                                            [methodology] ──▶ [experiments]
```

### 병렬 구현 가능 노드 그룹

| 그룹 | 노드들 | 예상 병렬 시간 |
|------|--------|---------------|
| **G1** | decay_estimation, gate_error_sum, critical_path | 2h |
| **G2** | kraus_operators, qiskit_integration | 3h |
| **G3** | qasmbench_import, revlib_import, custom_circuits | 2h |
| **G4** | json_export, csv_export, latex_table | 1h |

---

## 12. 테스트 커버리지 계획

### 12.1 단위 테스트 요구사항

| 원자 노드 | 테스트 케이스 | 커버리지 목표 |
|----------|--------------|--------------|
| `score_all_variants` | 빈 배열, 단일 변종, 100+ 변종 | 90% |
| `decay_estimation` | T1=0, T1=∞, T1<T2 (물리적 위반) | 95% |
| `gate_error_sum` | 0 게이트, 1000 게이트, 혼합 게이트 | 90% |
| `kraus_operators` | Identity, Full Depolarizing, Edge Probability | 95% |
| `parallel_evaluation` | 1 thread, N threads, 메모리 제한 | 85% |

### 12.2 통합 테스트

```rust
#[cfg(test)]
mod phase3_integration_tests {
    // 필수 통합 테스트 시나리오
    - test_full_optimization_pipeline()      // 전체 파이프라인
    - test_qiskit_comparison()               // Qiskit 비교
    - test_statistical_significance()        // 통계적 검증
    - test_scalability_20_qubits()          // 확장성
    - test_reproducibility()                 // 재현성
}
```

---

## 13. CI/CD 통합 계획

### 13.1 GitHub Actions 워크플로우

```yaml
# .github/workflows/phase3_benchmark.yml
name: Phase 3 Benchmark Suite

on:
  push:
    branches: [main, phase3-upgrade]
  schedule:
    - cron: '0 0 * * 0'  # 주간 벤치마크

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - run: cargo bench --bench phase3
      - run: python scripts/compare_qiskit.py
      - run: python scripts/generate_report.py
      - uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: results/
```

### 13.2 품질 게이트

| 게이트 | 조건 | 실패 시 조치 |
|--------|------|-------------|
| **테스트** | 전체 테스트 통과 | 머지 차단 |
| **커버리지** | ≥ 80% | 경고 |
| **성능** | 벤치마크 5% 이상 회귀 없음 | 머지 차단 |
| **Clippy** | 경고 0 | 머지 차단 |

---

## 14. Python 바인딩 계획 (qns_python)

### 14.1 Qiskit 연동을 위한 필수 API

```python
# qns_python 필수 구현 API
class QnsOptimizer:
    def from_qiskit_circuit(self, qc: QuantumCircuit) -> None
    def optimize(self, noise_model: NoiseModel) -> OptimizationResult
    def to_qiskit_circuit(self) -> QuantumCircuit

class NoiseModel:
    @staticmethod
    def from_ibm_backend(backend: IBMBackend) -> NoiseModel
    @staticmethod
    def from_t1t2(t1: float, t2: float, gate_error: float) -> NoiseModel
```

### 14.2 Gantree 추가 노드

```
L1_AlgorithmCompletion
    L2_PythonBindings // Python 바인딩 완성 (설계중)
        qiskit_circuit_convert // Qiskit 회로 변환 (원자)
        noise_model_convert // 노이즈 모델 변환 (원자)
        result_export // 결과 내보내기 (원자)
        type_stubs // .pyi 타입 스텁 (원자)
```

---

## 15. 추가 엣지 케이스

### 15.1 회로 관련

| 케이스 | 설명 | 예상 동작 |
|--------|------|----------|
| 모든 게이트 교환 가능 | H(0), H(1), H(2)... | 최대 변종 생성 |
| 순환 의존성 | CNOT(0,1), CNOT(1,2), CNOT(2,0) | 일부만 재정렬 |
| 측정 후 게이트 | Measure(0), H(0) | 측정 이후 게이트 보존 |
| 동일 큐빗 연속 게이트 | X(0), Y(0), Z(0) | 교환 불가 유지 |

### 15.2 노이즈 관련

| 케이스 | 설명 | 예상 동작 |
|--------|------|----------|
| T1 = 0 | 즉시 감쇠 | 최소 깊이 회로 선호 |
| T1 = T2 | 순수 위상 감쇠 없음 | 표준 최적화 |
| T2 > 2*T1 | 물리적 불가 | 에러 반환 또는 T2 클램핑 |
| 게이트 에러 = 1.0 | 완전 노이즈 | 게이트 최소화 |

### 15.3 성능 관련

| 케이스 | 설명 | 예상 동작 |
|--------|------|----------|
| 100+ 큐빗 | 대규모 회로 | 메모리 제한 내 동작 |
| 10,000+ 게이트 | 깊은 회로 | 타임아웃 내 완료 |
| 1M+ 변종 | 폭발적 탐색 공간 | Beam Search로 제한 |

---

## 16. 코드 품질 체크포인트

### 각 마일스톤별 품질 검증

#### M1: Algorithm Core

- [ ] 모든 스켈레톤 함수 구현됨
- [ ] 단위 테스트 80% 커버리지
- [ ] cargo clippy 경고 0

#### M2: Benchmark Ready

- [ ] Qiskit 비교 스크립트 동작
- [ ] 벤치마크 자동화 완료
- [ ] 결과 JSON 출력

#### M3: Validation Complete

- [ ] N=100 실험 완료
- [ ] p-value 계산 구현
- [ ] 신뢰구간 그래프 생성

#### M4: Paper Draft

- [ ] 논문 초안 완성
- [ ] 모든 그림/테이블 준비
- [ ] 재현성 패키지 테스트

---

## 17. 시니어 엔지니어 검토 결과

### 17.1 식별된 리스크 (추가)

| 리스크 | 확률 | 영향 | 대응 |
|--------|------|------|------|
| Python GIL 병목 | 중 | 중 | PyO3 release_gil 사용 |
| Qiskit 버전 호환성 | 높 | 중 | qiskit>=1.0 명시, CI 테스트 |
| 벤치마크 재현 불가 | 낮 | 높 | 시드 고정, Docker 이미지 |
| 메모리 OOM (대규모 회로) | 중 | 높 | 스트리밍 처리, 제한 설정 |

### 17.2 아키텍처 개선 권장사항

1. **스코어링 캐싱**: 동일 회로 재계산 방지
2. **점진적 탐색**: 전체 변종 생성 대신 필요 시 생성
3. **결과 직렬화**: 중간 결과 저장으로 재시작 지원
4. **로깅 레벨**: 벤치마크용 상세 로깅 옵션

---

*이 Gantree는 QNS 프로젝트의 Phase 3 저널 게재를 위한 로드맵입니다.*
*각 원자 노드는 15-30분 내 구현 가능하도록 분해되었습니다.*
*시니어 엔지니어 검토 완료: 2025-12-10*
