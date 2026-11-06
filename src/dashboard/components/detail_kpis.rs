pub fn render_detail_kpis_section() -> &'static str {
    r#"
<section id="detalle">
  <div class="section-head">
    <div>
      <h2>Auditoría Cuantitativa y Métricas por Bloque</h2>
    </div>
    <div class="strat-selector-wrap" style="display: flex; align-items: center; gap: 10px;">
      <label for="select-strat-detail" style="font-size: 12px; font-weight: 700; color: var(--mid);">Estrategia:</label>
      <select id="select-strat-detail" class="select-box">
        <!-- Opciones dinámicas -->
      </select>
    </div>
  </div>

  <!-- Control de Modo Drawdown -->
  <div class="ctrl-bar">
    <div class="grupo">
      <span class="badge-lbl">Modo Max Drawdown:</span>
      <div class="seg" id="d-ddm">
        <button type="button" data-v="pico" class="on">vs Capital Total (Pico)</button>
        <button type="button" data-v="cap">vs Capital Inicial</button>
      </div>
    </div>
    <span class="muted" id="d-ddm-desc">Max DD calculado respecto al capital total alcanzado en ese momento (pico previo).</span>
  </div>

  <!-- Pods Tripartitos de Métricas: IS, OOS, COMPLETO -->
  <div class="grid-3col" id="kpods">
    <!-- Generado dinámicamente: IS, OOS y COMPLETO con kgrid de 6 kpis cada uno -->
  </div>

  <!-- Desglose del Score SWAP Fitness -->
  <div class="pod" id="fitness-breakdown-pod" style="margin-top: 14px;">
    <div class="kpod-h">
      <h3>Desglose del Score SWAP Fitness (0 - 100)</h3>
      <span class="tag-badge">Fórmula SWAP EVO</span>
    </div>
    <div class="grid-3col" id="fitness-cards">
      <!-- 3 componentes de SWAP Fitness -->
    </div>
  </div>
</section>
"#
}
