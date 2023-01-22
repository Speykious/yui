#version 330 core

in vec2 v_uv;

out vec4 color;

uniform vec2 u_dimensions;
uniform float u_radius;
uniform float u_zoom;

uniform vec4 u_bgColor = vec4(1.0, 0.39, 0.46, 1.0);

uniform vec4 u_borderColor = vec4(1.0, 1.0, 1.0, 1.0);
uniform float u_strokeWidth = 20;

float roundRect(vec2 coords, float radius, vec2 dimensions, float zoom) {
  float mask = 1.0;

  if ((coords.x < 0.0 || coords.x > dimensions.x) ||
      (coords.y < 0.0 || coords.y > dimensions.y)) {
    // out of the rectangle's bounds

    mask = 0.0;
  } else if ((coords.x < radius || coords.x > dimensions.x - radius) &&
             (coords.y < radius || coords.y > dimensions.y - radius)) {
    // in the rounded corners

    vec2 corner = radius - min(coords, dimensions - coords);
    float dist = length(corner);
    mask = 1.0 - smoothstep(radius - 0.75 / zoom, radius + 0.75 / zoom, dist);
  }

  return mask;
}

void main() {
  vec2 coords = v_uv * u_dimensions;
  float rrMask = roundRect(coords, u_radius, u_dimensions, u_zoom);

  vec4 rrColor = u_bgColor;

  if (u_strokeWidth > 0.0) {
    vec2 inDimensions = u_dimensions - vec2(u_strokeWidth * 2);
    vec2 inCoords = coords - vec2(u_strokeWidth);
    float inRadius = u_radius - u_strokeWidth;
    float rrInner = roundRect(inCoords, inRadius, inDimensions, u_zoom);

    rrColor = mix(u_borderColor, u_bgColor, rrInner);
  }

  color = vec4(rrColor.rgb, rrColor.a * rrMask);
}