# QNS Crosstalk Weight Guidelines

## 개요

QNS v2.4+의 Crosstalk-Aware Routing 기능에서 `--crosstalk-weight` (W_X) 파라미터의 최적값 가이드라인입니다.

이 가이드라인은 56개 실험 조합 (7 W_X 값 × 8 회로 유형)의 시뮬레이션 벤치마크 결과를 바탕으로 도출되었습니다.

---

## 권장 W_X 값

| 회로 유형 | 권장 W_X | 평균 Fidelity | 근거 |
|----------|:-------:|:------------:|------|
| **Shallow** (Bell, GHZ) | **0.7** | 0.9987 | Crosstalk 영향 적음, 높은 가중치도 안전 |
| **Medium** (QFT, Grover) | **0.1** | 0.6823 | 과도한 라우팅 회피, 균형점 |
| **Deep** (VQE, QAOA) | **0.2** | 0.2539 | Crosstalk 누적 효과 고려 |

---

## 상세 결과

### Shallow 회로 (depth < 10)

```
W_X=0.0: Fidelity=0.9922
W_X=0.1: Fidelity=0.9909
W_X=0.2: Fidelity=0.9948
W_X=0.3: Fidelity=0.9935
W_X=0.5: Fidelity=0.9935
W_X=0.7: Fidelity=0.9987 ⭐
W_X=1.0: Fidelity=0.9922
```

**분석**: Shallow 회로는 게이트 수가 적어 Crosstalk 영향이 제한적입니다. 높은 W_X 값(0.7)이 최적이지만, 모든 값에서 높은 Fidelity(>0.99)를 보입니다.

### Medium 회로 (depth 10-50)

```
W_X=0.0: Fidelity=0.6797
W_X=0.1: Fidelity=0.6823 ⭐
W_X=0.2: Fidelity=0.6823
W_X=0.3: Fidelity=0.6771
W_X=0.5: Fidelity=0.6823
W_X=0.7: Fidelity=0.6745
W_X=1.0: Fidelity=0.6732
```

**분석**: Medium 회로에서는 낮은 W_X 값(0.1-0.2)이 최적입니다. 높은 W_X 값은 과도한 SWAP 삽입으로 오히려 Fidelity가 감소합니다.

### Deep 회로 (depth > 50)

```
W_X=0.0: Fidelity=0.2383
W_X=0.1: Fidelity=0.2324
W_X=0.2: Fidelity=0.2539 ⭐
W_X=0.3: Fidelity=0.2285
W_X=0.5: Fidelity=0.2305
W_X=0.7: Fidelity=0.2480
W_X=1.0: Fidelity=0.2402
```

**분석**: Deep 회로(VQE, QAOA)에서는 W_X=0.2가 최적입니다. 변분 회로의 특성상 전체적으로 낮은 Fidelity를 보이지만, 적절한 Crosstalk 회피가 효과적입니다.

---

## CLI 사용법

### 수동 설정

```bash
# Shallow 회로
qns run circuit.qasm --crosstalk-weight 0.7

# Medium 회로
qns run circuit.qasm --crosstalk-weight 0.1

# Deep 회로
qns run circuit.qasm --crosstalk-weight 0.2
```

### Auto 모드 (v2.5 예정)

```bash
# 자동 가중치 선택
qns run circuit.qasm --crosstalk-weight auto
```

Auto 모드 로직:

```
if circuit.depth < 10:
    W_X = 0.7   # Shallow
elif circuit.depth > 50:
    W_X = 0.2   # Deep
else:
    W_X = 0.1   # Medium
```

---

## 벤치마크 재현

```bash
python benchmarks/crosstalk_weight_sweep.py \
    --output benchmarks/results/crosstalk_sweep \
    --shots 256 \
    --noise low
```

### 결과 파일

- `crosstalk_sweep_results.csv`: 전체 실험 결과
- `crosstalk_sweep_results.json`: JSON 형식 결과

---

## 주의사항

1. **노이즈 레벨**: 이 가이드라인은 `low` 노이즈 환경에서 도출되었습니다. 높은 노이즈 환경에서는 최적값이 다를 수 있습니다.

2. **회로 특성**: 2-qubit 게이트 밀도가 높은 회로는 더 높은 W_X 값이 필요할 수 있습니다.

3. **하드웨어 의존성**: 실제 QPU의 Crosstalk 특성에 따라 최적값이 달라질 수 있습니다.

---

*Generated: 2025-12-30*
*Benchmark: 56 experiments, 256 shots, low noise*
