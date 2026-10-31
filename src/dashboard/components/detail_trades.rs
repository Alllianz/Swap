pub fn render_detail_trades_section() -> &'static str {
    r#"
<section id="rotaciones">
  <div class="section-head">
    <div>
      <h2>Registro Histórico de Rotaciones (Swaps Log)</h2>
      <p class="sub">Auditoría punto a punto de cada intercambio ejecutado por la estrategia activa. Inspecciona precios de salida y entrada, costo por comisiones, resultado de la rotación y el motivo cuantitativo de la decisión.</p>
    </div>
    <div class="trades-count-badge" id="trades-count-label">
      <!-- Total swaps -->
    </div>
  </div>

  <div class="pod">
    <div class="tabla-wrap">
      <table class="tabla" id="tabla-swaps">
        <thead>
          <tr>
            <th class="al-c"># Swap</th>
            <th class="al-l">Fecha</th>
            <th class="al-c">Activo Origen</th>
            <th class="al-c">→</th>
            <th class="al-c">Activo Destino</th>
            <th class="al-r">Precio Salida</th>
            <th class="al-r">Precio Entrada</th>
            <th class="al-r">PnL de Tenencia</th>
            <th class="al-r">Comisión</th>
            <th class="al-r">Capital Resultante</th>
            <th class="al-c">Días Sostenido</th>
            <th class="al-l">Motivo de Rotación</th>
          </tr>
        </thead>
        <tbody id="swaps-body">
          <!-- Generado dinámicamente -->
        </tbody>
      </table>
    </div>
  </div>
</section>
"#
}
