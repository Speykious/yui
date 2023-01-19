#version 330 core

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

out vec2 v_uv;

uniform mat4 u_mvp;

void main() {
  gl_Position = u_mvp * vec4(position, 0, 1);
  v_uv = uv;
}