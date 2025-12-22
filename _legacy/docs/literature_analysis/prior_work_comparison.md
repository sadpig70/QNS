# Prior Work Comparison: Noise-Aware NISQ Compilation

**작성일**: 2025-12-11
**목적**: QNS vs 선행연구 차별점 정량화 (npj QI 심사 대응)

---

## 1. 분석 대상 논문

| 논문 | 학회 | 연도 | 핵심 기여 |
|------|------|------|-----------|
| Murali et al. | ASPLOS | 2019 | Noise-adaptive compiler mappings |
| Tannu & Qureshi | ASPLOS | 2019 | Variability-aware qubit allocation |
| Li et al. | ASPLOS | 2019 | Qubit mapping for NISQ devices |

---

## 2. Murali et al. (ASPLOS 2019)

**제목**: "Noise-adaptive compiler mappings for noisy intermediate-scale quantum computers"

### 2.1 핵심 접근법

1. **Variation-Aware Qubit Movement (VQM)**
   - 큐비트 간 라우팅 시 에러율 고려
   - SWAP 경로 선택 시 노이즈 특성 반영

2. **Noise-Adaptive Mapping**
   - 캘리브레이션 데이터 기반 초기 매핑
   - 에러율 높은 큐빗/엣지 회피

3. **Reliability Metric**
   - PST (Probability of Successful Trial) 기반 스코어링
   - 회로 전체 성공 확률 추정

### 2.2 보고된 결과

- IBM 20-qubit (Johannesburg) 하드웨어 실험
- 평균 2.4× 개선 (PST 기준)
- 최대 18× 개선 (특정 회로)

### 2.3 QNS와의 차이점

| 항목 | Murali et al. | QNS |
|------|---------------|-----|
| **주요 전략** | 노이즈 회피 (Avoid) | 노이즈 활용 (Route-Through) |
| **최적화 대상** | 큐비트 선택 + 라우팅 | **엣지 선택 (Placement)** |
| **스코어링** | PST (곱) | Fidelity + Decay |
| **라우팅** | SWAP 최소화 | 품질 우선 SWAP |
| **구현** | Qiskit 기반 | **독립 Rust 엔진** |
| **오픈소스** | 비공개 | **MIT/Apache 공개** |
| **런타임** | 미보고 | **< 1ms** |

---

## 3. Tannu & Qureshi (ASPLOS 2019)

**제목**: "Not all qubits are created equal: A case for variability-aware policies for NISQ-era quantum computers"

### 3.1 핵심 접근법

1. **Variability Characterization**
   - 큐비트 간 에러율 편차 분석
   - 시간에 따른 변동성 측정

2. **Variability-Aware Allocation**
   - 고품질 큐비트 우선 할당
   - 에러율 낮은 서브그래프 선택

3. **Time-Varying Adaptation**
   - 캘리브레이션 주기 분석
   - 드리프트 대응 전략

### 3.2 보고된 결과

- IBM 16-qubit (Melbourne) 분석
- 5× 에러율 변동 관찰
- 변동성 인식 할당으로 2× 개선

### 3.3 QNS와의 차이점

| 항목 | Tannu & Qureshi | QNS |
|------|-----------------|-----|
| **초점** | 큐비트 품질 변동 | **엣지 품질 활용** |
| **전략** | 최고 품질 선택 | **최적 라우팅** |
| **기여** | 변동성 분석 | **Ablation 기반 정량화** |
| **실험** | 하드웨어 측정 | 시뮬레이션 (하드웨어 예정) |

---

## 4. Li et al. (ASPLOS 2019)

**제목**: "Tackling the qubit mapping problem for NISQ-era quantum devices"

### 4.1 핵심 접근법

1. **SABRE Algorithm**
   - SWAP-based BFS routing
   - Bidirectional search 최적화

2. **Initial Mapping Heuristics**
   - 회로 구조 기반 초기 배치
   - 2큐빗 게이트 빈도 분석

3. **Look-ahead Window**
   - 미래 게이트 고려한 SWAP 선택
   - 지역 최적화 회피

### 4.2 보고된 결과

- IBM/Google 토폴로지 시뮬레이션
- 깊이 50% 감소 (vs naive)
- SWAP 40% 감소

### 4.3 QNS와의 차이점

| 항목 | Li et al. (SABRE) | QNS |
|------|-------------------|-----|
| **목표 지표** | Circuit depth, SWAP count | **Fidelity** |
| **노이즈 인식** | 없음 (구조 기반) | **엣지별 에러율 반영** |
| **배치 전략** | 휴리스틱 | **품질 기반 최적화** |
| **Qiskit 통합** | 기본 제공 | 독립 구현 |

---

## 5. QNS 고유 기여점 (Novel Contributions)

### 5.1 Route-Through-Better-Edges

기존 연구가 "나쁜 엣지 회피"에 집중한 반면, QNS는 "좋은 엣지로 라우팅"에 집중:

```
기존: 에러율 10% 엣지 → 회피, 다른 경로 탐색
QNS:  에러율 1% 엣지 → 적극 활용, 배치 재조정
```

### 5.2 Ablation-Based Quantification

최초로 컴포넌트별 기여도를 정량화:

| 컴포넌트 | 기여도 |
|----------|--------|
| Placement | **74.1%** |
| Scoring | 14.3% |
| Reordering | 11.6% |

→ 컴파일러 개발 우선순위 제시

### 5.3 Sub-millisecond Runtime

| 방법 | 런타임 | 비고 |
|------|--------|------|
| Murali | 미보고 | Qiskit 의존 |
| SABRE | ~10ms | 복잡한 회로 |
| **QNS** | **< 1ms** | Rust 네이티브 |

### 5.4 Open-Source Implementation

- **언어**: Rust (메모리 안전, 고성능)
- **바인딩**: Python (PyO3)
- **라이선스**: MIT/Apache-2.0
- **저장소**: github.com/qns-ai/qns-mvp

---

## 6. 정량적 비교 요약

| 방법 | 접근법 | Fidelity 개선 | SWAP 오버헤드 | 런타임 | 코드 공개 |
|------|--------|---------------|---------------|--------|-----------|
| Murali | 큐비트 선택 | 2.4× (PST) | - | - | ❌ |
| Tannu | 변동성 인식 | 2× | - | - | ❌ |
| SABRE | 깊이 최적화 | 간접 | -40% | ~10ms | ✅ (Qiskit) |
| **QNS** | **엣지 라우팅** | **+58 pp** | 0* | **< 1ms** | **✅ Rust** |

*배치 최적화만 수행 시 추가 SWAP 불필요

---

## 7. Discussion에 추가할 문단

```markdown
### Comparison with Prior Noise-Aware Approaches

Several works have addressed noise-aware compilation for NISQ devices.
Murali et al. [11] introduced variation-aware qubit movement (VQM) that
considers error rates during SWAP routing, achieving 2.4× improvement in
success probability on IBM hardware. Tannu and Qureshi [10] characterized
qubit variability and proposed variability-aware allocation policies.
Li et al. [12] developed the SABRE algorithm focusing on circuit depth
and SWAP count minimization.

Our work differs in three key aspects: (1) We optimize for edge utilization
rather than qubit selection—actively routing operations through high-fidelity
edges rather than merely avoiding low-fidelity ones. (2) We provide the first
quantitative ablation analysis showing that placement optimization alone
contributes ~74% of achievable fidelity gains. (3) Our open-source Rust
implementation achieves sub-millisecond optimization times, enabling real-time
integration into compilation pipelines.
```

---

## 8. 한계 인정 (Limitations to Acknowledge)

1. **하드웨어 검증 부재**: 현재 시뮬레이션 기반 (vs Murali의 하드웨어 실험)
2. **토폴로지 일반성**: Heavy-hex 구현 완료, 하드웨어 테스트 예정
3. **비교 조건**: 동일 회로/하드웨어에서 직접 비교 필요

---

*문서 끝 - T5.1 선행연구 분석 완료*
