pub fn render_leaderboard_section() -> &'static str {
    r#"
<section id="leaderboard">
  <div class="section-head">
    <div>
      <h2>Leaderboard</h2>
    </div>
  </div>

  <!-- Podio Top 3 -->
  <div class="podio" id="lb-podio">
    <!-- Generado dinámicamente -->
  </div>

  <div class="pod">
    <div class="tabla-wrap">
      <table class="tabla" id="tabla-leaderboard">
        <thead>
          <tr>
            <th class="al-l">#</th>
            <th class="al-l">Activo / Estrategia</th>
            <th class="al-r">Score IS</th>
            <th class="al-r">Score OOS</th>
            <th class="al-r">Score COMP</th>
            <th class="al-r" title="Retención Score OOS / IS">Ret %</th>
            <th class="al-r" title="Retención normalizada por cantidad de velas IS vs OOS">Ret por velas</th>
            <th class="al-r">Retorno Total</th>
            <th class="al-r">Retorno OOS</th>
            <th class="al-r">vs B&H (BTC)</th>
            <th class="al-r" title="Max Drawdown en período OOS">Max DD OOS</th>
            <th class="al-r">Max Estanc.</th>
            <th class="al-r">Swaps</th>
            <th class="al-r">Sharpe</th>
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
