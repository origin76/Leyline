这是一个非常好的学习路线，而且和 Rust 很契合。
因为 SAT/SMT solver 属于“理论 + 系统实现 + 工程优化”三者高度结合的领域，你会同时学到：

* 逻辑与自动机
* 数据结构与增量算法
* 编译器/IR 风格设计
* 性能工程（cache、arena、watch literals）
* API 与约束表达
* incremental solving / assumptions / backtracking

而且 Rust 的 ownership 模型非常适合：

* arena allocator
* immutable term DAG
* typed AST
* region-based rollback
* persistent structures

---

# 先说结论：推荐路线

不要先做“漂亮 API”。

先做：

> SAT core → CDCL → DIMACS → SMT AST → DPLL(T) → theory solver → incremental API

也就是说：

```text
最小可工作的求解核心
    ↓
可 debug / 可 benchmark
    ↓
表达层(AST/API)
    ↓
SMT theory
    ↓
工程化
```

这是因为：

* solver 的 architecture 会反向决定 API
* 如果先做 DSL/API，后面 core 设计一变，整个接口会崩
* SMT solver 最难的是：

  * state management
  * backtracking
  * propagation architecture

这些必须先稳定

---

# 我建议的学习顺序（非常重要）

---

# Phase 0：不要立刻写 SMT

先只做 SAT。

很多人一开始就：

* quantifier
* EUF
* bitvector
* z3-like api

最后死于 complexity explosion。

真正的 SMT solver 本质是：

```text
CDCL SAT
+
Theory propagation
+
Theory conflict learning
```

SAT 是发动机。

---

# Phase 1：实现一个最小 DPLL SAT solver

目标：

支持：

```text
CNF
unit propagation
backtracking
```

输入：

DIMACS CNF

例如：

```text
p cnf 3 2
1 -2 0
2 3 0
```

数据结构：

```rust
type Var = u32;

struct Lit {
    var: Var,
    neg: bool,
}
```

不要一开始搞 fancy enum。

---

你会学到：

* CNF representation
* implication
* decision level
* assignment trail

---

# Phase 2：升级到 CDCL（关键）

这是 SAT solver 的真正现代核心。

实现：

* watched literals
* implication graph
* conflict analysis
* clause learning
* non-chronological backtracking
* VSIDS（后面再做）

这是整个项目最重要的阶段。

这里建议你大量参考：

* Decision Procedures
* Handbook of Satisfiability
* MiniSAT

MiniSAT 几乎是所有 solver 学习路线的圣经。

---

# 非常关键：先做“trail architecture”

你最终会发现 solver 的核心不是 clause。

而是：

```text
trail
decision level
undo
propagation queue
reason clause
```

本质上是：

```text
可回滚状态机
```

这和数据库/WAL/编译器优化其实很像。

---

# Phase 3：重构成 clean SAT core

这时再开始抽象。

例如：

```rust
trait Propagator {
    fn propagate(&mut self, ctx: &mut Context) -> PropagationResult;
}
```

然后：

```text
BooleanPropagator
EUFPropagator
ArithmeticPropagator
```

这一步非常像 LLVM pass architecture。

---

# Phase 4：做 SMT AST（现在才做）

现在你才适合做：

```rust
enum Term {
    Bool(bool),
    Var(Symbol),

    And(Vec<TermId>),
    Or(Vec<TermId>),

    Eq(TermId, TermId),

    Add(TermId, TermId),
}
```

注意：

## 不要 recursive Box AST

真正 solver 基本都是：

```text
interned DAG
arena allocated nodes
```

原因：

* sharing
* hashing
* equality
* canonicalization

Rust 非常适合：

```rust
slotmap
generational-arena
lasso
```

---

# Phase 5：实现 DPLL(T)

这时：

SAT solver 管：

```text
Boolean skeleton
```

theory solver 管：

```text
x + y > 3
f(a) = b
```

结构：

```text
SAT assigns literals
    ↓
theory checks consistency
    ↓
conflict clause returned
```

这是 SMT 的核心。

---

# 先做哪个 theory？

推荐顺序：

## 第一：

EUF（Equality with Uninterpreted Functions）

因为它最“纯”：

```text
f(a) = f(b)
a = b
```

核心算法：

```text
union find + congruence closure
```

这是 SMT 入门最佳 theory。

---

第二：

LIA（Linear Integer Arithmetic）

但这个会难很多。

你会涉及：

* simplex
* cuts
* bounds propagation

---

# Rust 架构建议（非常关键）

---

# 1. arena everything

不要：

```rust
Box<Term>
Rc<Term>
```

而是：

```rust
type TermId = u32;
Vec<TermNode>
```

solver 太 graph-heavy。

arena 是王道。

---

# 2. trail-based rollback

不要 clone state。

你应该：

```rust
trail.push(Assignment)
```

然后：

```rust
backtrack(level)
```

恢复。

SMT solver 本质就是：

```text
append-only log + rollback
```

---

# 3. separation of concerns

非常推荐：

```text
frontend/
    parser
    AST

core/
    SAT

theory/
    EUF
    arithmetic

ir/
    term storage

solver/
    orchestration
```

---

# 4. 先 single-threaded

千万不要：

```text
async
parallel CDCL
lock-free
```

你会死。

现代 solver 最重要的是：

* cache locality
* branch predictability

不是线程。

---

# 学习资料路线（按顺序）

---

## 第一阶段（SAT）

### MiniSAT 源码

一定要读。

即使是 C++。

---

## 第二阶段（理论）

### Decision Procedures

这是 SMT 圣经。

非常适合“自己实现”。

---

## 第三阶段（工业实现）

读：

* Z3
* cvc5

但不要一开始读。

否则会被 architecture 吓死。

---

# 关于 API：什么时候做？

我的建议：

## 非常晚。

前期：

```text
DIMACS
SMT-LIB2 parser
```

足够。

因为：

工业 solver 真正的“API”其实是：

```text
SMT-LIB2
```

而不是 fluent DSL。

---

# 项目命名建议

我建议不要叫：

```text
rustsat
rsmt
```

太 generic。

solver 名字最好：

* 短
* 硬核
* 有“推理/逻辑/石头/火焰”感
* 最好 2~3 syllables

---

我给你一些风格不同的名字：

---

## 偏 SMT/逻辑感

* **Axiom**
* **Taut**
* **Clause**
* **Resol**
* **Herbrand**
* **Skolem**
* **Davis**
* **PLL**
* **Trail**

---

## 偏 Rust 风格

* **FerrumSAT**
* **Oxide**
* **Crucible**
* **Anvil**
* **Forge**
* **Kiln**

---

## 我比较推荐的几个

### `Trail`

因为：

* SAT solver 本质就是 trail
* 很简洁
* 工业感强

---

### `Anvil`

感觉很好：

```text
constraints hammered into consistency
```

---

### `Skolem`

非常逻辑学。

---

### `Resol`

来自 resolution。

非常 SAT。

---

# 我最推荐你的路线（最终版）

```text
1. DIMACS parser
2. naive DPLL
3. watched literals
4. CDCL
5. clause learning
6. VSIDS
7. restart
8. SMT-LIB parser
9. term DAG
10. DPLL(T)
11. EUF
12. arithmetic
13. incremental push/pop
14. assumptions
15. proof logging
```

这是非常“正确”的学习曲线。

而且做到第 10 步时，你已经会比绝大多数“看过 SMT paper 的人”更理解 SMT solver。
