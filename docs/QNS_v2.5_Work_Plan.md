# QNS v2.4 → v2.5 시뮬레이션 기반 작업 계획

> **제약 조건**: IBM 하드웨어 구독 만료 (2025-01-17)로 인해 시뮬레이션 환경에서만 작업 진행
> **작업 기간**: 2025-12-30 ~ 2025-01-17 (약 18일)

---

## 작업 우선순위 요약

| 순위 | 작업 | 예상 기간 | 상태 |
| :---: | :--- | :---: | :---: |
| 1 | Crosstalk 가중치 최적화 연구 | 3-4일 | [x] ✅ 완료 |
| 2 | ZNE 모듈 구현 | 5-7일 | [x] ✅ 완료 |
| 3 | Aer Noisy 벤치마크 확장 | 2-3일 | [x] ✅ 완료 |
| 4 | MPS 시뮬레이터 고도화 | 3-5일 | [x] ✅ 완료 |
| 5 | 문서화 및 테스트 보강 | 2-3일 | [ ] |

**⏸️ 보류**: Crosstalk QPU 실증 벤치마크 → 하드웨어 재구독 후 진행

---

## 1. Crosstalk 가중치 최적화 연구 [우선순위: 1]

### 1.1 목표

- 회로 유형별 W_X 최적값 도출 (시뮬레이션 기반)
- CLI `--crosstalk-weight auto` 모드 구현

### 1.2 실험 매트릭스

```
W_X Values: [0.0, 0.1, 0.2, 0.3, 0.5, 0.7, 1.0]

Circuit Types:
├── Shallow (depth < 10): Bell, GHZ-3, GHZ-5
├── Medium (depth 10-50): QFT-5, QFT-10, Grover-5
├── Deep (depth > 50): VQE-4, QAOA-4
└── High 2Q Density: SWAP-heavy, Full entanglement

Metrics per (Circuit, W_X):
├── Estimated Fidelity (Aer Noisy)
├── Gate Count (SWAP 삽입 수)
├── Circuit Depth
└── Compilation Time
```

### 1.3 구현 태스크

```
[ ] benchmarks/crosstalk_weight_sweep.py 작성
[ ] 7x8 실험 매트릭스 실행 (56 조합)
[ ] Pareto 최적점 분석
[ ] CLI auto 모드 구현 (qns_cli/src/main.rs)
[ ] docs/QNS_Weight_Guidelines.md 작성
```

### 1.4 예상 결과

| 회로 유형 | 권장 W_X | 근거 |
| :--- | :---: | :--- |
| Shallow | 0.1-0.2 | Crosstalk 영향 적음, 과도한 라우팅 방지 |
| Medium | 0.3-0.5 | 균형점 |
| Deep/High-2Q | 0.5-0.7 | Crosstalk 누적 효과 큼 |

### 1.5 CLI 업데이트 제안

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

## 2. ZNE 통합 [우선순위: 2]

### 2.1 목표

- Zero-Noise Extrapolation 통합으로 에러 완화 기능 추가
- QNS의 "noise symbiosis" 철학 확장

### 2.2 아키텍처 설계

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

### 2.3 구현 단계

```
Phase 1: Core ZNE (3일)
├── [ ] LocalFolding 노이즈 증폭
│       └── CNOT → CNOT-CNOT†-CNOT
├── [ ] Linear Extrapolation
│       └── E(0) = 2*E(1) - E(2)
└── [ ] 기본 CLI 통합
        └── qns run --zne linear

Phase 2: Advanced ZNE (2일)
├── [ ] Richardson Extrapolation
├── [ ] Exponential Extrapolation
└── [ ] Adaptive scale factor 선택

Phase 3: QNS 통합 (2일)
├── [ ] Crosstalk + ZNE 결합
├── [ ] 자동 파이프라인
└── [ ] 벤치마크 업데이트
```

### 2.4 예상 API

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

### 2.5 검증 계획 (시뮬레이션)

| 회로 | Aer Noisy (ZNE 없음) | Aer Noisy (ZNE 적용) | 목표 개선 |
| :--- | :---: | :---: | :---: |
| VQE-4 | ~0.46 | ? | ≥ 0.55 (+20%) |
| QAOA-4 | TBD | TBD | ≥ 15% |
| Bell | ~1.0 | ~1.0 | 유지 |

---

## 3. Aer Noisy 벤치마크 확장 [우선순위: 3]

### 3.1 목표

- 다양한 노이즈 프로파일에서 QNS 성능 검증
- IBM 캘리브레이션 데이터 활용 노이즈 시뮬레이션

### 3.2 태스크

```
[ ] 노이즈 프로파일 다양화 (Low/Medium/High noise)
[ ] 회로 규모 확장 (10q, 15q, 20q)
[ ] Qiskit L3 vs QNS 비교 자동화
[ ] 결과 시각화 스크립트 (matplotlib)
[ ] docs/QNS_Simulation_Benchmark_Report.md 작성
```

### 3.3 노이즈 프로파일 정의

| 프로파일 | T1 | T2 | 1Q Error | 2Q Error |
| :--- | :---: | :---: | :---: | :---: |
| Low | 200μs | 150μs | 0.1% | 0.5% |
| Medium | 100μs | 80μs | 0.5% | 1.5% |
| High | 50μs | 30μs | 1.0% | 3.0% |

---

## 4. MPS 시뮬레이터 고도화 [우선순위: 4]

### 4.1 목표

- 대규모 회로 (30+ qubits) 시뮬레이션 성능 개선
- 메모리 효율성 향상

### 4.2 태스크

```
[ ] Bond dimension 적응형 조절
[ ] SVD truncation 최적화
[ ] 벤치마크: 30q QFT 시뮬레이션
[ ] StateVector vs MPS 성능 비교 문서화
```

---

## 5. 문서화 및 테스트 보강 [우선순위: 5]

### 5.1 태스크

```
[ ] API 문서 업데이트 (rustdoc)
[ ] 예제 회로 추가 (examples/)
[ ] 통합 테스트 확장
[ ] README.md 업데이트
[ ] CHANGELOG.md 작성
```

---

## 타임라인

```
Week 1 (12/30 - 01/05):
├── Crosstalk 가중치 최적화 연구 완료
└── ZNE 모듈 Phase 1 시작

Week 2 (01/06 - 01/12):
├── ZNE 모듈 Phase 1-2 완료
├── Aer Noisy 벤치마크 확장
└── MPS 시뮬레이터 고도화

Week 3 (01/13 - 01/17):
├── ZNE Phase 3 완료
├── 문서화 및 테스트 보강
└── v2.5 릴리스 준비
```

---

## 산출물 체크리스트

```
[ ] benchmarks/crosstalk_weight_sweep.py
[ ] crates/qns_zne/src/lib.rs
[ ] crates/qns_zne/src/noise_amplifier.rs
[ ] crates/qns_zne/src/extrapolator.rs
[ ] crates/qns_zne/src/factory.rs
[ ] docs/QNS_Weight_Guidelines.md
[ ] docs/QNS_Simulation_Benchmark_Report.md
[ ] docs/QNS_Technical_Specification_v2.5.md
```

---

*Updated: 2025-12-30*
*Author: Jung Wook Yang*
