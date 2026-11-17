@group(0) @binding(0) var<storage, read> data: array<f32>;
@group(0) @binding(1) var<storage, read_write> result: array<f32>;

@group(0) @binding(2) var<storage, read> weights: array<f32>;

fn dense(agent: u32, params: u32, base: u32, n: u32, m: u32, x: array<f32, 32>) -> array<f32, 32> {
    var y: array<f32, 32>;
    for (var i = 0u; i < m; i++) {
        var sum = weights[(agent/16u)*params*16u+(base+n*m+i)*16u+agent%16u];
        for (var j = 0u; j < n; j++) {
            sum += weights[(agent/16u)*params*16u+(base+i*n+j)*16u+agent%16u] * x[j];
        }
        y[i] = sum;
    }
    return y;
}

fn activate(x: array<f32, 32>, n: u32) -> array<f32, 32> {
    var y = x;
    for (var i = 0u; i < n; i++) { y[i] = tanh(x[i]); }
    return y;
}

fn load(base: u32, n: u32) -> array<f32, 32> {
    var x: array<f32, 32>;
    for (var i = 0u; i < n; i++) { x[i] = data[base+i]; }
    return x;
}

@compute @workgroup_size(64)
fn small(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= arrayLength(&data)/6u) { return; }
    let b = id.x*6u;
    let h = activate(dense(id.x, 83u, 0u, 6u, 8u, load(b, 6u)), 8u);
    let y = activate(dense(id.x, 83u, 56u, 8u, 3u, h), 3u);
    for (var i = 0u; i < 3u; i++) { result[id.x*20u+i] = y[i]; }
}

@compute @workgroup_size(64)
fn deep(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= arrayLength(&data)/16u) { return; }
    let b = id.x*16u;
    let h = activate(dense(id.x, 1140u, 0u, 16u, 32u, load(b, 16u)), 32u);
    let h2 = activate(dense(id.x, 1140u, 544u, 32u, 16u, h), 16u);
    let y = activate(dense(id.x, 1140u, 1072u, 16u, 4u, h2), 4u);
    for (var i = 0u; i < 4u; i++) { result[id.x*20u+i] = y[i]; }
}

@compute @workgroup_size(64)
fn recurrent(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= arrayLength(&data)/32u) { return; }
    let b = id.x*32u;
    var x = load(b, 32u);
    let z = dense(id.x, 1652u, 0u, 32u, 16u, x);
    let r = dense(id.x, 1652u, 528u, 32u, 16u, x);
    for (var i = 0u; i < 16u; i++) { x[16u+i] *= 1.0/(1.0+exp(-r[i])); }
    let c = dense(id.x, 1652u, 1056u, 32u, 16u, x);
    var h: array<f32, 32>;
    for (var i = 0u; i < 16u; i++) {
        let gate = 1.0/(1.0+exp(-z[i]));
        h[i] = (1.0-gate)*data[b+16u+i] + gate*tanh(c[i]);
        result[id.x*20u+4u+i] = h[i];
    }
    let y = activate(dense(id.x, 1652u, 1584u, 16u, 4u, h), 4u);
    for (var i = 0u; i < 4u; i++) { result[id.x*20u+i] = y[i]; }
}
