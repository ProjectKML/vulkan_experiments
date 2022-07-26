#version 460

#extension GL_NV_mesh_shader : require

layout(local_size_x = 1) in;
layout(triangles, max_vertices = 3, max_primitives = 1) out;

layout(location = 0) out vec3[] outColors;

void main() {
    gl_PrimitiveCountNV = 1;

    gl_MeshVerticesNV[0].gl_Position = vec4(-0.5, -0.5, 0.0, 1.0);
    gl_MeshVerticesNV[1].gl_Position = vec4(0.5, -0.5, 0.0, 1.0);
    gl_MeshVerticesNV[2].gl_Position = vec4(0.0, 0.5, 0.0, 1.0);

    outColors[0] = vec3(1.0, 0.0, 0.0);
    outColors[1] = vec3(0.0, 1.0, 0.0);
    outColors[2] = vec3(0.0, 0.0, 1.0);

    gl_PrimitiveIndicesNV[0] = 0;
    gl_PrimitiveIndicesNV[1] = 1;
    gl_PrimitiveIndicesNV[2] = 2;
}