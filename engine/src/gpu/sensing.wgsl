struct Params { width: f32, height: f32, cols: u32, rows: u32, count: u32, cells: u32, pad0: u32, pad1: u32 }
@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var<storage, read> agents: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> foods: array<vec2<f32>>;
@group(0) @binding(3) var<storage, read> grid: array<u32>;
@group(0) @binding(4) var<storage, read_write> result: array<f32>;
fn wrap_cell(x: i32, n: i32) -> u32 {
    let r = x - i32(floor(f32(x) / f32(n))) * n;
    return u32(r);
}
fn delta(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    let size = vec2<f32>(p.width, p.height);
    let d = b - a + size * 0.5;
    return d - floor(d / size) * size - size * 0.5;
}
fn search(i: u32, neighbors: bool) -> array<f32, 5> {
    let a = agents[i];
    let cell_size = vec2<f32>(p.width / f32(p.cols), p.height / f32(p.rows));
    let lo = vec2<i32>(floor((a.xy - vec2<f32>(a.z)) / cell_size));
    let hi = vec2<i32>(floor((a.xy + vec2<f32>(a.z)) / cell_size));
    let nx = select(min(hi.x - lo.x + 1, i32(p.cols)), i32(p.cols), p.pad0 != 0u);
    let ny = select(min(hi.y - lo.y + 1, i32(p.rows)), i32(p.rows), p.pad0 != 0u);
    var best = a.z * a.z;
    var nearest = 0xffffffffu;
    var direction = vec2<f32>(0.0);
    var count = 0u;
    let offset = select(0u, p.cells + 1u, neighbors);
    for (var y = 0; y < ny; y++) {
        for (var x = 0; x < nx; x++) {
            let cell = wrap_cell(select(lo.y, 0, p.pad0 != 0u) + y, i32(p.rows)) * p.cols + wrap_cell(select(lo.x, 0, p.pad0 != 0u) + x, i32(p.cols));
            for (var k = grid[offset + cell]; k < grid[offset + cell + 1u]; k++) {
                let j = grid[k];
                var pos: vec2<f32>;
                if neighbors {
                    if j == i || agents[j].w == 0.0 { continue; }
                    pos = agents[j].xy;
                } else { pos = foods[j]; }
                let d = delta(a.xy, pos);
                let d2 = dot(d, d);
                if d2 >= a.z * a.z { continue; }
                count++;
                if d2 < best || (d2 == best && j < nearest) {
                    best = d2; nearest = j; direction = d;
                }
            }
        }
    }
    let found = nearest != 0xffffffffu;
    let distance = sqrt(best);
    var unit = vec2<f32>(0.0);
    if found && distance > 0.000001 { unit = direction / distance; }
    var normalized = 1.0;
    if found && a.z > 0.0 { normalized = clamp(distance / a.z, 0.0, 1.0); }
    let scale = select(10.0, 5.0, neighbors);
    return array<f32, 5>(normalized, unit.x, unit.y, f32(count)/(f32(count)+scale), select(0.0, 1.0, found));
}
@compute @workgroup_size(64)
fn sense(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= p.count { return; }
    let f = search(i, false);
    var n = array<f32, 5>(1.0, 0.0, 0.0, 0.0, 0.0);
    if p.pad1 != 0u { n = search(i, true); }
    for (var k = 0u; k < 5u; k++) { result[i*9u+k] = f[k]; }
    for (var k = 0u; k < 4u; k++) { result[i*9u+5u+k] = n[k]; }
}
