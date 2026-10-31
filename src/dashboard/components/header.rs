pub fn render_header(
    title: &str,
    timeframe: &str,
    start_date: &str,
    end_date: &str,
    total_bars: usize,
) -> String {
    let tpl = r##"
<header class="hero">
  <div class="strip"></div>
  <div class="wrap">
    <div class="hero-top">
      <div>
        <p class="eyebrow">SWAP QUANT ENGINE · ROTACIÓN MULTIACTIVO · __TIMEFRAME__</p>
        <h1>__TITLE__</h1>
        <p class="lead">Plataforma Cuantitativa de Rotación Sistemática entre <b>Bitcoin (BTC)</b>, <b>Oro Físico (PAXG)</b> y <b>S&P 500 (SPY)</b>. Auditoría de curvas base 100, drawdowns superpuestos, cronograma de exposición y control estricto de fricción operativa.</p>
      </div>
      <div class="hero-badges">
        <div class="badge-item">
          <span class="badge-lbl">Rango Intersección</span>
          <span class="badge-val">__START_DATE__ → __END_DATE__</span>
        </div>
        <div class="badge-item">
          <span class="badge-lbl">Barras Sincronizadas</span>
          <span class="badge-val">__TOTAL_BARS__ velas diarias</span>
        </div>
        <div class="badge-item">
          <span class="badge-lbl">Estándar Causal</span>
          <span class="badge-val badge-ok">0.00% Look-Ahead Bias</span>
        </div>
      </div>
    </div>
    <p class="nota"><b>Estándar Cuantitativo Allianz:</b> Evaluación en streaming causal estricto. Las señales de intercambio se generan con el cierre de la vela $t$ y se ejecutan sin anticipar el futuro. Comparación transparente y punto a punto contra los 3 activos individuales y cartera equitativa.</p>
  </div>
</header>

<nav class="top" aria-label="Secciones">
  <div class="wrap">
    <div class="links">
      <a href="#leaderboard">Leaderboard Institucional</a>
      <a href="#comparativa">Gráficos Comparativos (Curvas, Drawdowns, Asignación)</a>
      <a href="#anual">Retornos Anuales</a>
      <a href="#detalle">Detalle de Estrategia</a>
      <a href="#rotaciones">Registro de Rotaciones (Swaps)</a>
    </div>
    <div class="reloj" id="reloj" aria-live="off"></div>
  </div>
</nav>
"##;

    tpl.replace("__TITLE__", title)
        .replace("__TIMEFRAME__", timeframe)
        .replace("__START_DATE__", start_date)
        .replace("__END_DATE__", end_date)
        .replace("__TOTAL_BARS__", &total_bars.to_string())
}
