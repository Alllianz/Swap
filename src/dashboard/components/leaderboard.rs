pub fn render_leaderboard_section() -> &'static str {
    r#"
<section id="leaderboard">
  <div class="section-head">
    <div>
      <h2>Leaderboard Comparativo Institucional (Estrategias vs Activos)</h2>
      <p class="sub">Auditoría simultánea de rendimiento de las estrategias cuantitativas de SWAP frente al Buy & Hold pasivo de los 3 activos (BTC, ORO, SPY) y la cartera balanceada (1/3 cada uno).</p>
    </div>
  </div>

  <div class="pod">
    <div class="tabla-wrap">
      <table class="tabla" id="tabla-leaderboard">
        <thead>
          <tr>
            <th class="al-l">Activo / Estrategia</th>
            <th class="al-l">Arquitectura / Tipo</th>
            <th class="al-r">Retorno Total</th>
            <th class="al-r">CAGR</th>
            <th class="al-r">Sharpe</th>
            <th class="al-r">Sortino</th>
            <th class="al-r">Max Drawdown</th>
            <th class="al-r">Calmar</th>
            <th class="al-r">SWAP Fitness</th>
            <th class="al-r">Rotaciones</th>
            <th class="al-r">Comisiones</th>
            <th class="al-c">Exposición Cartera (BTC | ORO | SPY | CASH)</th>
          </tr>
        </thead>
        <tbody id="leaderboard-body">
          <!-- Generado dinámicamente por JS desde el payload JSON -->
        </tbody>
      </table>
    </div>
  </div>
</section>
"#
}
