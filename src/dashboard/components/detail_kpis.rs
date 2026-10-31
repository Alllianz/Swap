pub fn render_detail_kpis_section() -> &'static str {
    r#"
<section id="detalle">
  <div class="section-head">
    <div>
      <h2>Auditoría Detallada de la Estrategia Seleccionada</h2>
      <p class="sub">Desglose exhaustivo de métricas de calidad de retorno, consistencia de crecimiento y distribución de exposición.</p>
    </div>
    <div class="strat-selector-wrap">
      <label for="select-strat-detail">Seleccionar Estrategia:</label>
      <select id="select-strat-detail" class="select-box">
        <!-- Opciones dinámicas -->
      </select>
    </div>
  </div>

  <div class="grid-kpis" id="kpis-container">
    <!-- Bloques de tarjetas KPI generados dinámicamente -->
  </div>

  <div class="pod" id="fitness-breakdown-pod">
    <div class="kpod-h">
      <h3>Desglose Institucional del Score SWAP Fitness (0 - 100)</h3>
      <span class="tag-badge">Fórmula Institucional SWAP</span>
    </div>
    <div class="grid-3col" id="fitness-cards">
      <!-- 3 componentes de SWAP Fitness -->
    </div>
  </div>
</section>
"#
}
