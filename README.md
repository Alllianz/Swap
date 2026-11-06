# SWAP Quant Engine 🔄📈

**SWAP** es una plataforma cuantitativa institucional de rotación sistemática y dinámica de capital entre tres activos macroeconómicos principales:
- **Bitcoin (BTC)**: Alta beta, liquidez global y expansión monetaria.
- **Oro Físico (PAXG / Gold)**: Cobertura inflacionaria, desdolarización y refugio ante estagflación.
- **S&P 500 (SPY)**: Renta variable institucional y crecimiento económico estadounidense.
- **Dólar / USDT (Cash)**: Preservación estricta de capital en fases de pánico financiero y shock sistémico.

---

## 🏛️ Estándar Cuantitativo Allianz
1. **0% Look-Ahead Bias**: Toda toma de decisión es estrictamente causal. La señal para el día $t$ se evalúa exclusivamente con datos cerrados hasta $t$.
2. **Evaluación Out-Of-Sample y Validación Cruzada**: Partición explícita de la intersección histórica real (2020 - 2026, 2,222 barras) con 70% In-Sample (IS) y 30% Out-Of-Sample (OOS).
3. **Métrica Institucional SWAP Fitness**:
   - **50%**: Ratio Ganancia Neta / Máximo Drawdown Nominal.
   - **25%**: Linealidad $R^2$ Compuesta ($0.40 R^2_{\text{global}} + 0.35 R^2_{\text{avg}} + 0.25 R^2_{\text{min}}$).
   - **25%**: Simetría de Pendiente (Slope Ratio IS/OOS) que penaliza drásticamente el sobreajuste.

---

## 🚀 Arquitecturas Implementadas

### Ruta A: SWAP GBDT Tri-Activo
Modelo basado en árboles potenciados por gradiente (*Gradient Boosted Decision Trees*) con inferencia causal punto a punto, extrayendo 16 características relativas y clasificando el activo con mayor probabilidad de alfa.

### Ruta B: SWAP Dual Momentum Sistemático
Combina:
- **Relative Momentum**: Selección del activo con mejor desempeño relativo en ventanas multitemporales (28 y 84 días).
- **Absolute Trend Filter**: Regla de corte absoluto (70 días) para migrar a Cash o refugio ante caídas sistémicas.
- **Vol-Targeting**: Normalización por volatilidad histórica (ATR).
- **Gobernanza Anti-Churn**: Margen de histéresis de 1.5% y mínimo de 5-7 días de tenencia para evitar desgaste por comisiones.

### Ruta C: SWAP Macro Regime Hybrid
Árbol de decisión de regímenes macroeconómicos:
- *Risk-On Growth* (SPY > SMA200 y BTC fuerte) $\to$ BTC.
- *Steady Growth Equities* (SPY alcista con baja volatilidad) $\to$ S&P 500.
- *Stagflation / Hedging* (Oro batiendo a bonos/acciones) $\to$ PAX Gold.
- *Panic / Systemic Shock* $\to$ CASH (USDT).

---

## 🌐 Dashboards Visuales Multiactivo y de Optimización
El motor genera dashboards HTML interactivos standalone en la carpeta `dashboard/`:
- **`dashboard_A.html`**: Auditoría completa de la optimización de la **Ruta A (SWAP GBDT ML)**, incluyendo todos los candidatos evaluados clasificados por SWAP Fitness Score (con el ganador en 1° puesto) frente a los benchmarks de mercado.
- **`dashboard_B.html`**: Auditoría completa de la optimización de la **Ruta B (SWAP Dual Momentum)** con todas las configuraciones evaluadas.
- **`dashboard_C.html`**: Auditoría completa de la optimización de la **Ruta C (SWAP Híbrido de Regímenes Macroeconómicos)**.
- **`dashboard_comparativo.html`**: Comparativa simultánea directa de las **3 mejores configuraciones optimizadas (Ganador A vs Ganador B vs Ganador C)** frente a los 4 benchmarks pasivos (BTC, Oro, S&P 500 y Cartera Equitativa 1/3).
- **`dashboard_swap_1d.html`**: Dashboard general multiactivo.

Cada dashboard incluye:
- **C1 · Curvas en Base 100**: Comparativa interactiva conmutable en escala logarítmica/lineal contra Buy & Hold de BTC, ORO, SPY y Cartera Equitativa (1/3 cada uno).
- **C2 · Drawdown Superpuesto**: Comparación de caídas máximas frente al -77% histórico de BTC.
- **C3 · Cronograma de Asignación (Timeline)**: Mapa visual temporal de en qué activo estuvo posicionado el capital día por día.
- **C4 · Matriz Anual (Heatmap)**: Rentabilidad año a año (2020 a 2026) y alfa generado.
- **Leaderboard y Registro de Rotaciones**: Auditoría completa de cada swap, precios, comisiones y motivos de rotación.

---

## 🛠️ Requisitos y Ejecución

Compilado al 100% en Rust:

```bash
# Compilar en modo release
cargo build --release

# Ejecutar terminal interactiva de SWAP (Opción [7] genera automáticamente los 4 dashboards)
cargo run --release
```
