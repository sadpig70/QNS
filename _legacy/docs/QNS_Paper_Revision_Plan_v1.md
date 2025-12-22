# QNS 논문 수정 작업 계획서 (Gantree)

**Version**: 1.2
**작성일**: 2025-12-11
**목표**: npj Quantum Information 심사 지적사항 대응
**범위**: 지적사항 2-5번 (하드웨어 검증 제외)

---

## 작업 범위 정의

| 번호 | 지적 사항 | 상태 |
|------|-----------|------|
| 1 | 실제 하드웨어 검증 | 🔒 보류 (다음 달 진행) |
| 2 | SOTA Baseline 비교 (Qiskit Sabre/Noise-aware) | 📋 본 계획 |
| 3 | 토폴로지 확장 (Heavy-hex/Grid) | 📋 본 계획 |
| 4 | 수치/수식 정합성 수정 | 📋 본 계획 |
| 5 | Murali et al. (ASPLOS 2019) 차별점 정량화 | 📋 본 계획 |

---

## Gantree 작업 트리

```
QNS_Paper_Revision // 논문 수정 전체 작업 (설계중)
    T2_SOTA_Baseline_Comparison // Qiskit 등 SOTA와 비교 실험 (설계중)
        T2.1_Qiskit_Integration // Qiskit 연동 환경 구축 (설계중)
            T2.1.1_Setup_Qiskit_Environment // Python venv + qiskit 설치 (설계중)
            T2.1.2_Create_Qiskit_Transpiler_Wrapper // Sabre/DenseLayout 래퍼 구현 (설계중)
            T2.1.3_Implement_NoiseAware_Layout // noise-aware initial_layout 옵션 구현 (설계중)
        T2.2_Benchmark_Suite_Design // 비교 벤치마크 스위트 설계 (설계중)
            T2.2.1_Select_Representative_Circuits // GHZ, QFT, VQE, QAOA 회로 선정 (설계중)
            T2.2.2_Define_Metrics // Fidelity, SWAP count, Depth, Time 지표 정의 (설계중)
            T2.2.3_Create_Comparison_Script // 자동화된 비교 스크립트 작성 (설계중)
        T2.3_Run_Comparison_Experiments // 비교 실험 실행 (설계중)
            T2.3.1_Run_Qiskit_Default // optimization_level=0,1,2,3 실행 (설계중)
            T2.3.2_Run_Qiskit_NoiseAware // noise-aware layout 실행 (설계중)
            T2.3.3_Run_QNS_Optimizer // QNS 최적화 실행 (설계중)
            T2.3.4_Collect_Results // 결과 수집 및 CSV/JSON 저장 (설계중)
        T2.4_Generate_Comparison_Figures // 비교 그래프 생성 (설계중)
            T2.4.1_Create_BarChart_Fidelity // Fidelity 비교 막대그래프 (설계중)
            T2.4.2_Create_Table_AllMetrics // 전체 지표 비교 테이블 (설계중)
            T2.4.3_Export_LaTeX_Tables // LaTeX 형식 테이블 내보내기 (설계중)
    T3_Topology_Extension // Heavy-hex/Grid 토폴로지 확장 (설계중)
        T3.1_Implement_HeavyHex_Topology // IBM Heavy-hex 구현 (분해)
        T3.2_Implement_Grid_Topology // Google Sycamore Grid 구현 (분해)
        T3.3_Update_Router_Algorithm // 라우터 알고리즘 토폴로지 대응 (설계중)
            T3.3.1_Add_SWAP_Insertion_Logic // SWAP 게이트 삽입 로직 추가 (설계중)
            T3.3.2_Implement_A_Star_Routing // A* 기반 라우팅 구현 (설계중)
            T3.3.3_Update_Scoring_For_SWAP // SWAP 비용 반영 스코어링 (설계중)
        T3.4_HeavyHex_Benchmarks // Heavy-hex 벤치마크 실행 (설계중)
            T3.4.1_Create_IBM_127q_Profile // IBM Eagle 127큐빗 프로파일 (설계중)
            T3.4.2_Run_HeavyHex_Experiments // Heavy-hex 실험 실행 (설계중)
            T3.4.3_Generate_HeavyHex_Figures // 결과 Figure 생성 (설계중)
        T3.5_Grid_Benchmarks // Grid 토폴로지 벤치마크 (설계중)
            T3.5.1_Create_Google_54q_Profile // Sycamore 54큐빗 프로파일 (설계중)
            T3.5.2_Run_Grid_Experiments // Grid 실험 실행 (설계중)
            T3.5.3_Generate_Grid_Figures // 결과 Figure 생성 (설계중)
    T4_Numerical_Consistency_Fix // 수치/수식 정합성 수정 (설계중)
        T4.1_Fix_Fidelity_Calculations // Fidelity 계산 수정 (설계중)
            T4.1.1_Fix_WorstEdge_0Percent // 0% → 정확한 계산값(~35%) 수정 (설계중)
            T4.1.2_Add_Decoherence_Explanation // Decoherence 효과 설명 추가 (설계중)
            T4.1.3_Update_Table1_Values // Table 1 수치 업데이트 (설계중)
        T4.2_Clarify_Improvement_Metrics // 개선율 표현 명확화 (설계중)
            T4.2.1_Use_PercentagePoints // "90% improvement" → "+90pp" 표기 (설계중)
            T4.2.2_Add_Average_Improvement // 전체 평균 개선율 추가 (설계중)
            T4.2.3_Update_Abstract // Abstract 문구 수정 (설계중)
        T4.3_Fix_Model_Consistency // 모델 수식 일관성 (설계중)
            T4.3.1_Align_Analytical_MonteCarlo // Analytical vs MC 모델 정렬 (설계중)
            T4.3.2_Add_T1_Term_To_Formula // Fidelity 공식에 T1 항 추가 (설계중)
            T4.3.3_Document_Approximations // 근사 가정 명시 (설계중)
        T4.4_Fix_Ablation_Rounding // Ablation 비율 반올림 수정 (설계중)
            T4.4.1_Recalculate_Percentages // 합계 100% 되도록 재계산 (설계중)
            T4.4.2_Add_Rounding_Note // 반올림 설명 추가 (설계중)
        T4.5_Fix_Delta_Statistics // Δ 통계 수정 (설계중)
            T4.5.1_Report_MaxDelta // 최대 Δ 값 명시 (GHZ 11.59%) (설계중)
            T4.5.2_Explain_GHZ_Outlier // GHZ 케이스 차이 원인 설명 (설계중)
    T5_Murali_Differentiation // Murali et al. 차별점 정량화 (설계중)
        T5.1_Literature_Analysis // 선행 연구 심층 분석 (설계중)
            T5.1.1_Analyze_Murali_ASPLOS2019 // Murali et al. 논문 상세 분석 (설계중)
            T5.1.2_Analyze_Tannu_ASPLOS2019 // Tannu & Qureshi 논문 분석 (설계중)
            T5.1.3_Analyze_Li_ASPLOS2019 // Li et al. 논문 분석 (설계중)
            T5.1.4_Create_Comparison_Matrix // 방법론 비교 매트릭스 작성 (설계중)
        T5.2_Identify_Novel_Contributions // QNS 고유 기여점 도출 (설계중)
            T5.2.1_Document_RouteThruBetterEdges // Route-Through-Better-Edges 차별점 (설계중)
            T5.2.2_Document_AblationMethodology // Ablation Study 방법론 차별점 (설계중)
            T5.2.3_Document_RuntimeAdvantage // Sub-ms 런타임 장점 (설계중)
            T5.2.4_Document_OpenSource_Value // 오픈소스 구현 가치 (설계중)
        T5.3_Quantitative_Comparison // 정량적 비교 실험 (설계중)
            T5.3.1_Cite_Murali_Results // Murali 논문 결과 인용 + 동일 조건 QNS 측정 (설계중)
            T5.3.2_Run_HeadToHead_Benchmark // 동일 조건 비교 실험 (설계중)
            T5.3.3_Generate_Comparison_Table // 비교 테이블 생성 (설계중)
            T5.3.4_Statistical_Significance_Test // 통계적 유의성 검정 (설계중)
        T5.4_Update_Paper_Sections // 논문 섹션 업데이트 (설계중)
            T5.4.1_Expand_RelatedWork // Related Work 섹션 확장 (설계중)
            T5.4.2_Strengthen_Discussion // Discussion 차별점 강화 (설계중)
            T5.4.3_Add_Comparison_Table // 비교 테이블 추가 (설계중)
    T6_Paper_Document_Update // 논문 문서 최종 업데이트 (설계중)
        T6.1_Update_Results_Section // Results 섹션 업데이트 (설계중)
        T6.2_Update_Methods_Section // Methods 섹션 업데이트 (설계중)
        T6.3_Update_Supplementary // Supplementary 자료 업데이트 (설계중)
        T6.4_Final_Review // 최종 리뷰 및 교정 (설계중)
```

---

## 분해 노드 상세 (T3.1, T3.2)

### T3.1_Implement_HeavyHex_Topology // IBM Heavy-hex 구현 (분해)

```
T3.1_Implement_HeavyHex_Topology // IBM Heavy-hex 토폴로지 구현 (설계중)
    T3.1.1_Study_HeavyHex_Structure // Heavy-hex 구조 분석 (설계중)
        T3.1.1.1_Document_IBM_Eagle_Layout // IBM Eagle 레이아웃 문서화 (설계중)
        T3.1.1.2_Identify_Edge_Patterns // 엣지 패턴 식별 (설계중)
    T3.1.2_Implement_HeavyHex_Generator // Heavy-hex 생성기 구현 (설계중)
        T3.1.2.1_Create_HexCell_Unit // 단위 Hex 셀 구현 (설계중)
        T3.1.2.2_Implement_Cell_Tiling // 셀 타일링 알고리즘 (설계중)
        T3.1.2.3_Add_Bridge_Qubits // 브릿지 큐빗 추가 (설계중)
    T3.1.3_Add_HeavyHex_To_HardwareProfile // HardwareProfile에 추가 (설계중)
        T3.1.3.1_Implement_HeavyHex_Factory // heavy_hex() 팩토리 메서드 (설계중)
        T3.1.3.2_Add_EdgeFidelity_Map // 엣지별 fidelity 맵 추가 (설계중)
        T3.1.3.3_Write_Unit_Tests // 단위 테스트 작성 (설계중)
```

### T3.2_Implement_Grid_Topology // Google Sycamore Grid 구현 (분해)

```
T3.2_Implement_Grid_Topology // Google Sycamore Grid 토폴로지 구현 (설계중)
    T3.2.1_Study_Grid_Structure // Grid 구조 분석 (설계중)
        T3.2.1.1_Document_Sycamore_Layout // Sycamore 레이아웃 문서화 (설계중)
        T3.2.1.2_Identify_Coupling_Pattern // 결합 패턴 식별 (설계중)
    T3.2.2_Implement_Grid_Generator // Grid 생성기 구현 (설계중)
        T3.2.2.1_Create_NxM_Grid // N×M 그리드 생성 (설계중)
        T3.2.2.2_Add_Diagonal_Couplings // 대각선 결합 추가 (옵션) (설계중)
        T3.2.2.3_Handle_Missing_Qubits // 결손 큐빗 처리 (설계중)
    T3.2.3_Add_Grid_To_HardwareProfile // HardwareProfile에 추가 (설계중)
        T3.2.3.1_Implement_Grid_Factory // grid() 팩토리 메서드 (설계중)
        T3.2.3.2_Add_EdgeFidelity_Map // 엣지별 fidelity 맵 추가 (설계중)
        T3.2.3.3_Write_Unit_Tests // 단위 테스트 작성 (설계중)
```

---

## 작업 우선순위 및 의존성

```
┌─────────────────────────────────────────────────────────────────┐
│                    작업 의존성 그래프                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Phase 1 (병렬 진행 가능)                                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │     T4       │  │    T3.1      │  │    T3.2      │          │
│  │ 수치 정합성  │  │  Heavy-hex   │  │    Grid      │          │
│  │    수정      │  │    구현      │  │    구현      │          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
│         │                 │                 │                   │
│         │                 └────────┬────────┘                   │
│         │                          │                            │
│  Phase 2 (병렬 진행 가능)          ▼                            │
│  ┌──────────────┐         ┌──────────────┐                     │
│  │    T5.1      │         │    T3.3      │                     │
│  │ 선행연구분석 │         │ 라우터 업데이트│                     │
│  └──────┬───────┘         └──────┬───────┘                     │
│         │                        │                              │
│         └────────────┬───────────┘                              │
│  Phase 3             ▼                                          │
│         ┌──────────────┐  ┌──────────────┐                     │
│         │    T2.1      │  │   T5.2-T5.3  │                     │
│         │ Qiskit 연동  │  │ 차별점/비교  │                     │
│         └──────┬───────┘  └──────┬───────┘                     │
│                │                 │                              │
│  Phase 4       ▼                 │                              │
│         ┌──────────────┐         │                              │
│         │  T2.2-T2.4   │         │                              │
│         │ SOTA 비교    │         │                              │
│         └──────┬───────┘         │                              │
│                │                 │                              │
│                └────────┬────────┘                              │
│  Phase 5                ▼                                       │
│                ┌──────────────┐                                 │
│                │  T5.4 + T6   │                                 │
│                │ 논문 최종화  │                                 │
│                └──────────────┘                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 원자화 노드 예시 (15-30분, <50줄)

### T4.1.1_Fix_WorstEdge_0Percent

**목표**: Table 1의 "0.00%" fidelity를 정확한 계산값으로 수정

**현재 문제**:
```
Worst Edge: 10 CNOT(L₁,L₂) on 90% edge
본문 설명: 0.90¹⁰ ≈ 0.35 (35%)
Table 1 값: 0.00% ← 모순
```

**수정 작업**:
1. `scripts/recalculate_fidelity.py` 작성
2. Decoherence 포함 정확한 계산:
   ```
   F = (0.90)^10 × exp(-t_total/T2)
   t_total = 10 × 660ns = 6.6μs
   T2 = 80μs
   F ≈ 0.35 × 0.92 ≈ 0.32 (32%)
   ```
3. Table 1 업데이트: `0.00%` → `32.2%`
4. 본문 설명 수정: "near 0%" → "~32% (further degraded by coherence decay)"

**예상 시간**: 20분
**예상 코드**: ~30줄

---

### T4.2.1_Use_PercentagePoints

**목표**: "90% improvement" 표현을 "+90 percentage points"로 명확화

**수정 위치**:
1. Abstract: "up to 90% fidelity improvement" → "up to +70 percentage points (from ~25% to ~95%)"
2. Table 1 Improvement 열: "+90.00%" → "+90.0 pp" (percentage points)
3. Results 섹션: 일관된 용어 사용

**예상 시간**: 15분

---

### T3.1.3.1_Implement_HeavyHex_Factory

**목표**: `HardwareProfile::heavy_hex(n)` 팩토리 메서드 구현

**위치**: `crates/qns_core/src/types/hardware_profile.rs`

**구현 스펙**:
```rust
impl HardwareProfile {
    /// Creates IBM Heavy-hex topology
    /// n: number of unit cells (e.g., n=3 for 27 qubits)
    pub fn heavy_hex(n: usize) -> Self {
        // ... 구현
    }
}
```

**예상 시간**: 30분
**예상 코드**: ~50줄

---

### T5.1.1_Analyze_Murali_ASPLOS2019

**목표**: Murali et al. (ASPLOS 2019) 논문 핵심 방법론 분석

**분석 항목**:
1. 논문 제목: "Noise-adaptive compiler mappings for noisy intermediate-scale quantum computers"
2. 핵심 접근법:
   - Variation-aware qubit movement (VQM)
   - Noise-adaptive mapping
   - Reliability 기반 큐비트 선택
3. 차이점 식별:
   ```
   Murali et al.          vs      QNS
   ─────────────────────────────────────────
   큐비트 선택 중심        │   엣지 라우팅 중심
   정적 매핑              │   동적 재배선
   Reliability metric     │   Fidelity scoring
   하드웨어 실험 포함      │   시뮬레이션 기반
   ```

**산출물**: `docs/literature_analysis/murali_2019_analysis.md`

**예상 시간**: 30분

---

### T5.3.3_Generate_Comparison_Table

**목표**: QNS vs 선행연구 정량적 비교 테이블 생성

**테이블 구조**:
```
| 방법론 | 접근법 | Fidelity↑ | SWAP↓ | Time | 구현 |
|--------|--------|-----------|-------|------|------|
| Murali | 큐비트선택 | +X% | - | O(?) | 비공개 |
| Tannu  | 변동성인식 | +Y% | - | O(?) | 비공개 |
| QNS    | 엣지라우팅 | +70pp | 0 | O(n) | 오픈소스 |
```

**예상 시간**: 25분

---

## 분해 노드 상세 (T5.1)

### T5.1_Literature_Analysis // 선행 연구 심층 분석 (분해)

```
T5.1_Literature_Analysis // 선행 연구 심층 분석 (설계중)
    T5.1.1_Analyze_Murali_ASPLOS2019 // Murali et al. 상세 분석 (설계중)
        T5.1.1.1_Extract_Core_Algorithm // 핵심 알고리즘 추출 (설계중)
        T5.1.1.2_Identify_Assumptions // 가정 및 제약조건 식별 (설계중)
        T5.1.1.3_Document_Results // 보고된 결과 정리 (설계중)
    T5.1.2_Analyze_Tannu_ASPLOS2019 // Tannu & Qureshi 분석 (설계중)
        T5.1.2.1_Extract_Variability_Model // 변동성 모델 추출 (설계중)
        T5.1.2.2_Compare_With_QNS_Scoring // QNS 스코어링과 비교 (설계중)
    T5.1.3_Analyze_Li_ASPLOS2019 // Li et al. 분석 (설계중)
        T5.1.3.1_Extract_Mapping_Algorithm // 매핑 알고리즘 추출 (설계중)
        T5.1.3.2_Compare_Complexity // 복잡도 비교 (설계중)
    T5.1.4_Create_Comparison_Matrix // 비교 매트릭스 작성 (설계중)
        T5.1.4.1_Define_Comparison_Axes // 비교 축 정의 (설계중)
        T5.1.4.2_Fill_Matrix_Data // 매트릭스 데이터 채우기 (설계중)
        T5.1.4.3_Export_LaTeX_Table // LaTeX 테이블 생성 (설계중)
```

---

## 산출물 체크리스트

### Phase 1 완료 시

- [ ] T4: 수정된 논문 초안 (수치 정합성)
- [ ] T3.1: `HardwareProfile::heavy_hex()` 구현
- [ ] T3.2: `HardwareProfile::grid()` 구현
- [ ] 단위 테스트 통과

### Phase 2 완료 시

- [ ] T3.3: SWAP 삽입 포함 라우터
- [ ] T5.1: 선행연구 분석 문서 (`docs/literature_analysis/`)

### Phase 3 완료 시

- [ ] T2.1: Qiskit 연동 래퍼
- [ ] T5.2: QNS 고유 기여점 문서
- [ ] T5.3: 정량적 비교 실험 결과

### Phase 4 완료 시

- [ ] T2.2-T2.4: SOTA 비교 실험 결과
- [ ] 새로운 Figure/Table 생성
- [ ] `results/` 폴더에 CSV/JSON 저장

### Phase 5 완료 시

- [ ] T5.4: Related Work / Discussion 섹션 강화
- [ ] T6: 최종 논문 초안
- [ ] Supplementary 자료 업데이트
- [ ] 재현성 스크립트 정리

---

## 리스크 및 대응

| 리스크 | 영향 | 대응 방안 |
|--------|------|-----------|
| Qiskit API 변경 | 비교 실험 지연 | qiskit 버전 고정 (1.0.x) |
| Heavy-hex 구현 복잡도 | 일정 지연 | 단순화된 버전 먼저 구현 |
| SWAP 라우팅 정확도 | 결과 신뢰도 | A* 알고리즘 검증 테스트 |
| Murali 논문 접근 제한 | 비교 분석 불가 | arXiv 버전 활용, 인용 논문 참조 |
| 차별점 미약 판정 | 논문 reject | 복합 기여점(속도+오픈소스+ablation) 강조 |
| 선행연구 재구현 오류 | 비교 신뢰도 | 논문 수치 직접 인용, 재구현 명시 |

---

## 예상 일정

| Phase | 작업 | 예상 소요 |
|-------|------|----------|
| 1 | T4 수치수정 + T3.1/T3.2 토폴로지 | 2-3일 |
| 2 | T3.3 라우터 + T5.1 문헌분석 | 2-3일 |
| 3 | T2.1 Qiskit + T5.2/T5.3 차별점 | 3-4일 |
| 4 | T2.2-T2.4 SOTA 비교 | 2-3일 |
| 5 | T5.4 + T6 최종 문서화 | 2일 |
| **합계** | | **11-15일** |

---

*문서 끝 - Version 1.2 (자체 검토 완료)*
