#version 330 core
// No 'in' attributes needed!

// We pass this to the fragment shader
out vec2 v_texCoord;

void main() {
    // Hardcode the 3 vertices of a giant triangle
    vec2 vertices[3] = vec2[3](
        vec2(-1.0, -1.0), // Bottom-left
        vec2( 3.0, -1.0), // Far-right
        vec2(-1.0,  3.0)  // Far-top
    );

    // Use gl_VertexID to pick the vertex
    gl_Position = vec4(vertices[gl_VertexID], 0.0, 1.0);

    // Calculate texture coordinates (UVs) from the vertex position
    v_texCoord = (gl_Position.xy + 1.0) / 2.0;
}
