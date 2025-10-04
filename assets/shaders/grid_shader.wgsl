#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct GridMaterial {
    line_colour: vec4<f32>,
    line_width: f32,
    grid_size: f32,
    offset: vec2<f32>,
    resolution: vec2<f32>,
    grid_intensity: f32,
}

@group(2) @binding(0)
var<uniform> material: GridMaterial;

fn mod_vec2(x: vec2<f32>, y: f32) -> vec2<f32> {
    return x - y * floor(x / y);
}
fn grid(frag_coord: vec2<f32>, space: f32, grid_width: f32) -> f32 {
    let grid_pos = mod_vec2(frag_coord, space);
    let dist_to_edge = min(grid_pos, space - grid_pos);
    let min_dist = min(dist_to_edge.x, dist_to_edge.y);
    
    // Use screen-space derivatives for automatic anti-aliasing
    let fw = fwidth(min_dist);
    return 1.0 - smoothstep(grid_width - fw, grid_width + fw, min_dist);
}
@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // Convert UV to pixel coordinates using the mesh resolution
    let pixel_coord = mesh.uv * material.resolution + material.offset;
    
    // 2-size grid using grid_size uniform (now in pixel units)
    let fine_grid = grid(pixel_coord, material.grid_size, material.line_width);
    let coarse_grid = grid(pixel_coord, material.grid_size * 5.0, material.line_width * 2.0);
    
    // Combine grids
    let grid_factor = max(fine_grid, coarse_grid);
    
    // Apply gradient from center of mesh
    let p = pixel_coord;
    let c = material.resolution / 2.0;
    let gradient = (1.0 - length(c - p) / material.resolution.x * 0.7);
    
    // If we're on a grid line, show the line color, otherwise transparent
    if (grid_factor > 0.01) {
        let final_color = material.line_colour.rgb * gradient;
        let final_alpha = grid_factor * material.line_colour.a;
        return vec4<f32>(final_color, final_alpha);
    } else {
        // Transparent background
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}
