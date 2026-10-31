pub fn render_compare_charts_section() -> &'static str {
    r#"
<section id="comparativa">
  <div class="section-head">
    <div>
      <h2>Herramientas Visuales Multiactivo: Curvas, Drawdowns y Asignación</h2>
      <p class="sub">Visualización interactiva punto a punto. Compara el crecimiento de capital, profundidad de caídas y dinámica de rotación entre estrategias y activos subyacentes.</p>
    </div>
  </div>

  <div class="pod">
    <div class="ctrl-bar">
      <div class="grupo">
        <span>Escala de Curva:</span>
        <div class="seg" id="seg-escala">
          <button class="active" data-val="log">Logarítmica</button>
          <button data-val="linear">Lineal</button>
        </div>
      </div>
      <div class="grupo">
        <span>Ventana Temporal:</span>
        <div class="seg" id="seg-rango">
          <button class="active" data-val="todo">Todo el Período</button>
          <button data-val="is">In-Sample (IS)</button>
          <button data-val="oos">Out-Of-Sample (OOS)</button>
        </div>
      </div>
      <div class="grupo">
        <span>Estrategia Activa para Asignación:</span>
        <select id="select-strat-allocation" class="select-box">
          <!-- Opciones pobladas dinámicamente -->
        </select>
      </div>
    </div>

    <!-- Pestañas de Gráficos -->
    <div class="tabs" role="tablist">
      <button role="tab" class="tab-btn active" data-tab="tab-curvas">C1 · Curvas Base 100 vs BTC, ORO y SPY</button>
      <button role="tab" class="tab-btn" data-tab="tab-drawdowns">C2 · Drawdown Superpuesto</button>
      <button role="tab" class="tab-btn" data-tab="tab-asignacion">C3 · Cronograma de Asignación de Activos</button>
      <button role="tab" class="tab-btn" data-tab="tab-anual">C4 · Retorno Anual Comparativo</button>
    </div>

    <!-- PESTAÑA C1: CURVAS BASE 100 -->
    <div class="tab-panel active" id="tab-curvas">
      <div class="panel-header">
        <div class="panel-info">
          <h3>C1 · Evolución de Capital Normalizado a Base 100</h3>
          <p>Muestra cómo habría evolucionado \$100 invertidos en cada estrategia versus el Buy & Hold de Bitcoin, Oro Físico, S&P 500 y la Cartera Equitativa (1/3 c/u). Haz clic en la leyenda para ocultar/mostrar cualquier curva.</p>
        </div>
        <div class="legend-toggles" id="curvas-toggles">
          <!-- Checkboxes dinámicos -->
        </div>
      </div>
      <div class="chart-container chart-lg">
        <canvas id="chart-curvas"></canvas>
      </div>
    </div>

    <!-- PESTAÑA C2: DRAWDOWNS SUPERPUESTOS -->
    <div class="tab-panel" id="tab-drawdowns">
      <div class="panel-header">
        <div class="panel-info">
          <h3>C2 · Curvas de Drawdown Superpuestas</h3>
          <p>Evalúa el riesgo de cola y la protección de capital durante caídas de mercado severas (ej. criptoinvierno 2022 o correcciones bursátiles). Identifica si la estrategia amortigua o elimina las caídas profundas de BTC (-77%) o SPY (-25%).</p>
        </div>
      </div>
      <div class="chart-container chart-md">
        <canvas id="chart-drawdowns"></canvas>
      </div>
    </div>

    <!-- PESTAÑA C3: CRONOGRAMA DE ASIGNACIÓN -->
    <div class="tab-panel" id="tab-asignacion">
      <div class="panel-header">
        <div class="panel-info">
          <h3>C3 · Asignación Histórica de Cartera en el Tiempo</h3>
          <p>Muestra exactamente en qué activo estuvo posicionado el capital día a día: <span class="badge-dot dot-btc">BTC</span>, <span class="badge-dot dot-gold">ORO</span>, <span class="badge-dot dot-sp500">S&P 500</span> o <span class="badge-dot dot-cash">CASH (Liquidez)</span>.</p>
        </div>
      </div>
      <div class="chart-container chart-md">
        <canvas id="chart-asignacion"></canvas>
      </div>
    </div>

    <!-- PESTAÑA C4: RETORNO ANUAL -->
    <div class="tab-panel" id="tab-anual">
      <div class="panel-header">
        <div class="panel-info">
          <h3>C4 · Matriz de Retorno por Año Calendario</h3>
          <p>Desglose año a año comparando cada estrategia activa contra los 3 activos. Verde indica que la estrategia superó al activo de referencia en ese año particular.</p>
        </div>
      </div>
      <div class="tabla-wrap">
        <table class="tabla" id="tabla-anual">
          <thead>
            <tr>
              <th class="al-l">Año</th>
              <th class="al-l">Estrategia</th>
              <th class="al-r">Retorno Estrategia</th>
              <th class="al-r">Retorno BTC</th>
              <th class="al-r">Retorno ORO</th>
              <th class="al-r">Retorno S&P 500</th>
              <th class="al-r">Alfa vs BTC</th>
              <th class="al-r">Alfa vs S&P 500</th>
            </tr>
          </thead>
          <tbody id="anual-body">
            <!-- Dinámico -->
          </tbody>
        </table>
      </div>
    </div>

  </div>
</section>
"#
}
