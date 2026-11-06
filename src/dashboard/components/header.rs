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
  </div>
</header>

<nav class="top" aria-label="Secciones">
  <div class="wrap">
    <div class="links">
      <a href="#leaderboard">Leaderboard</a>
      <a href="#mano-a-mano">Comparador Mano a Mano</a>
      <a href="#comparativa">Gráficos Comparativos</a>
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
