<script lang=ts>
    import * as wasm from 'matex-gui'

    export let width: string | number = "100%";
    export let height: string | number = "100%";

    wasm.new_debug_app("donkey");
</script>

<div style:width={width} style:height={height}>
    <canvas id="donkey"></canvas>
</div>

<style>
    div {
        position: relative;
    }
    canvas {
        display: block;
        margin-right: auto;
        margin-left: auto;

        width: 100% !important;
        height: 100% !important;
        position: static !important; 
    }
</style>