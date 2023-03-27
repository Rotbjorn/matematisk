<script lang=ts>
    import { run } from "matex-web";
    import EguiCanvas from "../lib/components/EguiCanvas.svelte";


    let source_text: string;

    function onClick() {
        let output = run(source_text)
        alert(output)
    }
</script>


<h1>My nice little app</h1>
<textarea bind:value={source_text}/>
<button on:click={onClick}>RUN CODE</button>

<EguiCanvas />