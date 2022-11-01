import init, {
    init_threads,
    contribute_wasm,
    subgroup_check_wasm,
    get_pot_pubkeys_wasm,
} from "./pkg/wrapper_small_pot.js";

onmessage = async (event) => {
    const entropy = event.data;
    console.log("available threads:", navigator.hardwareConcurrency);

    await init();
    await init_threads(navigator.hardwareConcurrency);

    fetch('./initialContribution.json').then(response => {
        response.json().then(async (data) => {
            const json_string = JSON.stringify(data);
            let secret = await sha256(entropy);
            let identity = "eth|0x000000000000000000000000000000000000dead";

            console.log("get potPubkeys from entropy");
            const potPubkeys = get_pot_pubkeys_wasm(secret);
            console.log(potPubkeys);

            console.log("start contribution");
            const startTime = performance.now();
            const result_string = contribute_wasm(
                json_string,
                secret,
                identity,
            );
            const endTime = performance.now();
            const result = JSON.parse(result_string);
            console.log(result)
            console.log(`Contribution took ${endTime - startTime} milliseconds`);

            console.log("perform subgroups checks in previous and new contribution");
            // check initial contribution
            const checkInitialContribution = subgroup_check_wasm(json_string);
            console.log(checkInitialContribution)
            // check updated contribution
            const checkUpdatedContribution = subgroup_check_wasm(result_string);
            console.log(checkUpdatedContribution)
        });
    });
}

async function sha256(message) {
    // encode as UTF-8
    const msgBuffer = new TextEncoder().encode(message);
    // hash the message
    const hashBuffer = await crypto.subtle.digest('SHA-256', msgBuffer);
    // convert ArrayBuffer to Array
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    // convert bytes to hex string
    const hashHex = hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
    return hashHex;
}
