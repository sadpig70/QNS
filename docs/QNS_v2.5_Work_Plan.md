# QNS v2.4 → v2.5 권고사항 상세 작업 계획

## 1. Crosstalk QPU 실증 벤치마크 [우선순위: 최상]

### 1.1 목표
- v2.4 Crosstalk-Aware Routing의 실제 QPU 성능 검증
- arXiv/저널 투고용 실증 데이터 확보

### 1.2 실험 설계

| 항목 | 내용 |
|------|------|
| 대상 QPU | IBM Torino (133q) - 이미 검증된 접근권 활용 |
| 비교군 | Crosstalk OFF (W_X=0) vs Crosstalk ON (W_X=0.5) |
| 회로 | Bell, GHZ-5, QFT-5, VQE-4, QAOA-4 |
| 반복 | 회로당 최소 8192 shots × 5회 반복 |
| 측정 지표 | Fidelity, TVD, Success Probability |

### 1.3 실험 프로토콜

```
Phase 1: Baseline Collection
├── 각 회로를 Qiskit L3 transpile (crosstalk 무시)
├── IBM Torino 실행
└── 결과 저장: baseline_results.json

Phase 2: QNS Crosstalk-OFF
├── QNS 최적화 (--crosstalk-weight 0.0)
├── IBM Torino 실행
└── 결과 저장: qns_no_xtalk_results.json

Phase 3: QNS Crosstalk-ON
├── QNS 최적화 (--crosstalk-weight 0.5)
├── IBM Torino 실행
└── 결과 저장: qns_xtalk_results.json

Phase 4: 통계 분석
├── Paired t-test (Crosstalk ON vs OFF)
├── Effect size (Cohen's d)
└── 신뢰구간 95% 계산
```

### 1.4 예상 산출물

- benchmarks/crosstalk_qpu_validation.py
- results/crosstalk_qpu_results.json
- docs/QNS_Crosstalk_Validation_Report.md

### 1.5 성공 기준

| 지표 | 목표 |
|------|------|
| Fidelity 향상 | ≥ 3% (Crosstalk ON vs OFF) |
| 통계적 유의성 | p < 0.05 |
| 재현성 | 5회 반복 중 4회 이상 일관된 결과 |

---

## 2. Crosstalk 가중치 최적값 연구 [우선순위: 높음]

### 2.1 목표
- 회로 유형별 W_X 최적값 도출
- 사용자 가이드라인 제공

### 2.2 실험 매트릭스

```
W_X Values: [0.0, 0.1, 0.2, 0.3, 0.5, 0.7, 1.0]

Circuit Types:
├── Shallow (depth < 10): Bell, GHZ
├── Medium (depth 10-50): QFT, Grover
├── Deep (depth > 50): VQE, QAOA
└── High 2Q Density: SWAP-heavy, Full entanglement

Metrics per (Circuit, W_X):
├── Estimated Fidelity (시뮬레이션)
├── Gate Count (SWAP 삽입 수)
├── Circuit Depth
└── Compilation Time
```

### 2.3 분석 방법

```python
# Pareto 최적점 탐색
for circuit_type in circuit_types:
    results = []
    for w_x in [0.0, 0.1, 0.2, 0.3, 0.5, 0.7, 1.0]:
        fidelity = run_simulation(circuit, w_x)
        gate_count = count_gates(optimized_circuit)
        results.append((w_x, fidelity, gate_count))
    
    # Knee point detection
    optimal_w_x = find_knee_point(results)
    recommendations[circuit_type] = optimal_w_x
```

### 2.4 예상 산출물

| 회로 유형 | 권장 W_X | 근거 |
|----------|---------|------|
| Shallow | 0.1-0.2 | Crosstalk 영향 적음, 과도한 라우팅 방지 |
| Medium | 0.3-0.5 | 균형점 |
| Deep/High-2Q | 0.5-0.7 | Crosstalk 누적 효과 큼 |

### 2.5 CLI 업데이트 제안

```bash
# 자동 가중치 선택 모드
qns run circuit.qasm --crosstalk-weight auto

# 내부 로직
if circuit.depth < 10:
    w_x = 0.15
elif circuit.two_qubit_ratio > 0.4:
    w_x = 0.6
else:
    w_x = 0.35
```

---

## 3. ZNE 통합 로드맵 [우선순위: 중간]

### 3.1 목표
- Zero-Noise Extrapolation 통합으로 에러 완화 기능 추가
- QNS의 "noise symbiosis" 철학 확장

### 3.2 아키텍처 설계

```
qns/
├── crates/
│   ├── qns_zne/           # 🆕 신규 모듈
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── noise_amplifier.rs    # 노이즈 증폭 기법
│   │   │   ├── extrapolator.rs       # 외삽 알고리즘
│   │   │   └── factory.rs            # 증폭 회로 생성
│   │   └── Cargo.toml
```

### 3.3 구현 우선순위

```
Phase 1: Core ZNE (2주)
├── LocalFolding 노이즈 증폭
│   └── CNOT → CNOT-CNOT†-CNOT
├── Linear Extrapolation
│   └── E(0) = 2*E(1) - E(2)
└── 기본 CLI 통합
    └── qns run --zne linear

Phase 2: Advanced ZNE (2주)
├── Richardson Extrapolation
├── Exponential Extrapolation
└── Adaptive scale factor 선택

Phase 3: QNS 통합 (1주)
├── Crosstalk + ZNE 결합
├── 자동 파이프라인
└── 벤치마크 업데이트
```

### 3.4 예상 API

```rust
// qns_zne/src/lib.rs

pub struct ZneConfig {
    pub method: ExtrapolationMethod,  // Linear, Richardson, Exponential
    pub scale_factors: Vec<f64>,       // [1.0, 2.0, 3.0]
    pub folding_type: FoldingType,     // Local, Global
}

pub fn apply_zne(
    circuit: &CircuitGenome,
    config: &ZneConfig,
    executor: &dyn CircuitExecutor,
) -> f64 {
    // 1. Generate scaled circuits
    // 2. Execute each
    // 3. Extrapolate to zero noise
}
```

### 3.5 검증 계획

| 회로 | ZNE 없음 | ZNE 적용 | 목표 개선 |
|------|---------|---------|----------|
| VQE-4 | 0.46 | ? | ≥ 0.55 (+20%) |
| QAOA-4 | TBD | TBD | ≥ 15% |

---

## 4. 통합 타임라인

```
Week 1-2: Crosstalk QPU 실증 (§1)
├── 실험 스크립트 작성
├── IBM Torino 실행
└── 결과 분석 및 문서화

Week 3: 가중치 최적화 연구 (§2)
├── 시뮬레이션 매트릭스 실행
├── 최적값 도출
└── CLI auto 모드 구현

Week 4-6: ZNE 통합 (§3)
├── qns_zne 모듈 구현
├── 테스트 및 검증
└── 문서화

Week 7: v2.5 릴리스
├── 전체 벤치마크 업데이트
├── arXiv 논문 초안 업데이트
└── GitHub 릴리스
```

---

## 5. 파일 체크리스트

```
[ ] benchmarks/crosstalk_qpu_validation.py
[ ] benchmarks/crosstalk_weight_sweep.py
[ ] crates/qns_zne/src/lib.rs
[ ] crates/qns_zne/src/noise_amplifier.rs
[ ] crates/qns_zne/src/extrapolator.rs
[ ] docs/QNS_Crosstalk_Validation_Report.md
[ ] docs/QNS_Weight_Guidelines.md
[ ] QNS_Technical_Specification_v2_5.md
```

---

**핵심 우선순위:** §1 Crosstalk QPU 실증 → §2 가중치 최적화 → §3 ZNE

---

*Generated: 2025-12-22*
*Author: ClNeo (Claude) for Jung Wook Yang*
