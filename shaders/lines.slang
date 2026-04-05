struct Line {
  float3 start;
  float3 end;
  float3 colour;
};

struct DebugLineParams {
  float4x4 viewProj[2];
  float2 viewportSize;
};

[[vk::binding(0, 0)]]
StructuredBuffer<Line> gLines;

struct VSOut {
  float4 position : SV_Position;
  float3 colour : TEXCOORD0;
  float2 aPx : TEXCOORD1;
  float2 bPx : TEXCOORD2;
  float radiusPx : TEXCOORD3;
};

static const float LINE_WIDTH_PX = 3.0;

[[vk::push_constant]]
ConstantBuffer<DebugLineParams> gParams;

[shader("vertex")]
VSOut vs_main(uint vertexID: SV_VertexID, uint viewID: SV_ViewID, ) {
  VSOut o;

  uint lineIndex = vertexID / 6;
  uint localVert = vertexID % 6;

  Line line = gLines[lineIndex];

  float4 clipA = mul(gParams.viewProj[viewID], float4(line.start, 1.0));
  float4 clipB = mul(gParams.viewProj[viewID], float4(line.end, 1.0));

  // First-pass cheap sanity: if either endpoint is behind the camera,
  // just collapse the primitive. Later you can do proper near-plane clipping.
  if (clipA.w <= 0.0 || clipB.w <= 0.0) {
    o.position = float4(0.0, 0.0, 0.0, 0.0);
    o.colour = 0.0.xxx;
    o.aPx = 0.0.xx;
    o.bPx = 0.0.xx;
    o.radiusPx = 0.0;
    return o;
  }

  float2 ndcA = clipA.xy / clipA.w;
  float2 ndcB = clipB.xy / clipB.w;

  float2 aPx = ndc_to_px(ndcA, gParams.viewportSize);
  float2 bPx = ndc_to_px(ndcB, gParams.viewportSize);

  float2 ab = bPx - aPx;
  float len = length(ab);
  float2 dir = (len > 1e-5) ? (ab / len) : float2(1.0, 0.0);
  float2 nrm = float2(-dir.y, dir.x);

  float r = LINE_WIDTH_PX * 0.5;

  // Rectangle corners expanded by radius in both tangent and normal directions.
  float2 corners[4];
  corners[0] = aPx - dir * r - nrm * r; // start-left
  corners[1] = aPx - dir * r + nrm * r; // start-right
  corners[2] = bPx + dir * r - nrm * r; // end-left
  corners[3] = bPx + dir * r + nrm * r; // end-right

  static const uint indexLUT[6] = { 0, 1, 2, 2, 1, 3 };
  uint cornerIndex = indexLUT[localVert];
  float2 cornerPx = corners[cornerIndex];

  bool useA = (cornerIndex == 0 || cornerIndex == 1);
  float4 baseClip = useA ? clipA : clipB;

  float2 cornerNdc = px_to_ndc(cornerPx, gParams.viewportSize);

  o.position = float4(cornerNdc * baseClip.w, baseClip.z, baseClip.w);
  o.colour = line.colour;
  o.aPx = aPx;
  o.bPx = bPx;
  o.radiusPx = r;

  return o;
}

[shader("fragment")]
float4 fs_main(VSOut i) : SV_Target {
  float2 p = i.position.xy;

  float d = distance_to_segment(p, i.aPx, i.bPx);

  // Tiny bit of AA from screen-space distance.
  float aa = max(fwidth(d), 1.0);
  float alpha = 1.0 - smoothstep(i.radiusPx - aa, i.radiusPx + aa, d);

  if (alpha <= 0.0)
    discard;

  // Premultiplied is nice if you want it.
  return float4(i.colour * alpha, alpha);
}

float2 ndc_to_px(float2 ndc, float2 viewportSize) {
  return (ndc * 0.5 + 0.5) * viewportSize;
}

float2 px_to_ndc(float2 px, float2 viewportSize) {
  return (px / viewportSize) * 2.0 - 1.0;
}

float distance_to_segment(float2 p, float2 a, float2 b) {
  float2 ab = b - a;
  float denom = dot(ab, ab);

  if (denom < 1e-6)
    return length(p - a);

  float t = saturate(dot(p - a, ab) / denom);
  float2 q = a + t * ab;
  return length(p - q);
}
