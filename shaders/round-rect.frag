#version 330 core

in vec2 v_uv;

out vec4 color;

uniform vec2 u_dimensions;
uniform vec4 u_color = vec4(1, 0.39, 0.46, 1);
uniform float u_radius = 10;
uniform float u_zoom;

float roundRect(vec2 coords, float radius, vec2 dimensions, float zoom) {
  float mask = 1.0;

  if ((coords.x < radius || coords.x > dimensions.x - radius) &&
      (coords.y < radius || coords.y > dimensions.y - radius)) {

    float cornerx = radius - min(coords.x, dimensions.x - coords.x);
    float cornery = radius - min(coords.y, dimensions.y - coords.y);
    float dist = length(vec2(cornerx, cornery));

    mask = 1.0 - smoothstep(radius - 0.75 / zoom, radius + 0.75 / zoom, dist);
  }

  return mask;
}

void main() {
  color = u_color * roundRect(v_uv, u_radius, u_dimensions, u_zoom);
}