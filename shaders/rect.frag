#version 330 core

in vec2 v_uv;

out vec4 color;

uniform vec2 u_dimensions;

void main() {
  color = vec4(v_uv.x / u_dimensions.x, v_uv.y / u_dimensions.y, 0, 1);
}