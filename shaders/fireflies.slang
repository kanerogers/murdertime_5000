[[vk::binding(0, 0)]]
RWStructuredBuffer<Particle> particles;

[[vk::binding(0, 1)]]
ConstantBuffer<SimParams> simParams;

struct Particle {
  float3 position; // xyz = world position
  float size;      // size in metres
  float3 velocity; // xyz = velocity,
  float phase;     // pulse phase
  float3 color;    // rgb = base color
  float brightness;
};

struct VSOutput {
  float4 position : SV_Position;
  float2 localUV : TEXCOORD0; // -1..1 on the quad
  float4 color : TEXCOORD1;
  float phase : TEXCOORD2;
  float3 velocity : TEXCOORD3;
};

struct SimParams {
  float4x4 viewProj[2];
  float3 cameraRightWS[2];
  float3 cameraUpWS[2];

  float3 rightHandPos;    // right hand xyz pos
  float rightHandAttract; // right hand attract strength
  float3 leftHandPos;     // xyz pos
  float leftHandRepel;    // left hand repel strength
  float3 headPos;         // head xyz pos, w repel strength
  float headRepel;

  float dt;
  float time;
  float handRadius;
  float headRadius;

  float drag;
  float wanderStrength;
  float maxSpeed;
  float swirlStrength;

  uint particleCount;
};

static const float2 kCorners[4] = {
  float2(-1.0, -1.0),
  float2(1.0, -1.0),
  float2(-1.0, 1.0),
  float2(1.0, 1.0),
};

static const uint kIndices[6] = { 0, 1, 2, 2, 1, 3 };

float hash1(float n) { return frac(sin(n) * 43758.5453123); }

float3 hash3(float n) {
  return float3(hash1(n + 0.123), hash1(n + 4.567), hash1(n + 8.901)) * 2.0 -
         1.0;
}

float3 safeNormalize(float3 v) {
  float lenSq = dot(v, v);
  if (lenSq < 1e-8)
    return float3(0.0, 0.0, 0.0);
  return v * rsqrt(lenSq);
}

float3 attractForce(float3 particlePos, float3 targetPos, float radius,
                    float strength, float swirlStrength) {
  float3 toTarget = targetPos - particlePos;
  float dist = length(toTarget);

  if (dist < 0.0001 || dist > radius)
    return float3(0.0, 0.0, 0.0);

  float3 dir = toTarget / dist;
  float t = 1.0 - dist / radius;
  float falloff = t * t;

  float3 force = dir * (strength * falloff);

  // cheap sideways swirl around world-up
  float3 swirl = cross(dir, float3(0.0, 1.0, 0.0));
  float swirlLenSq = dot(swirl, swirl);
  if (swirlLenSq > 1e-8) {
    swirl *= rsqrt(swirlLenSq);
    force += swirl * (swirlStrength * falloff);
  }

  return force;
}

float3 repelForce(float3 particlePos, float3 sourcePos, float radius,
                  float strength) {
  float3 away = particlePos - sourcePos;
  float dist = length(away);

  if (dist < 0.0001 || dist > radius)
    return float3(0.0, 0.0, 0.0);

  float3 dir = away / dist;
  float t = 1.0 - dist / radius;
  float falloff = t * t;

  return dir * (strength * falloff);
}

[shader("compute")]
[numthreads(16, 1, 1)]
void main(uint3 dispatchThreadID: SV_DispatchThreadID) {
  uint i = dispatchThreadID.x;
  SimParams sim = simParams;
  if (i >= sim.particleCount)
    return;

  Particle p = particles[i];

  float3 pos = p.position;
  float3 vel = p.velocity;
  float phase = p.phase;

  float3 force = float3(0.0, 0.0, 0.0);

  // wandering
  float3 noise =
      safeNormalize(hash3(float(i) * 17.13 + floor(sim.time * 9.0) + phase));
  force += noise * sim.wanderStrength;

  // right hand attracts + swirls
  force += attractForce(pos, sim.rightHandPos.xyz, sim.handRadius,
                        sim.rightHandAttract, sim.swirlStrength);

  // left hand repels
  force +=
      repelForce(pos, sim.leftHandPos.xyz, sim.handRadius, sim.leftHandRepel);

  // keep them out of your skull
  force += repelForce(pos, sim.headPos.xyz, sim.headRadius, sim.headRepel);

  // drag
  force += -vel * sim.drag;

  // integrate
  vel += force * sim.dt;

  float speed = length(vel);
  if (speed > sim.maxSpeed) {
    vel *= sim.maxSpeed / speed;
  }

  pos += vel * sim.dt;

  // slow pulse phase
  phase += sim.dt * (0.5 + 0.25 * hash1(float(i) * 1.37));

  p.position = pos;
  p.velocity = vel;
  p.phase = phase;

  particles[i] = p;
}

[shader("vertex")]
VSOutput vs_main(uint vertexID: SV_VertexID, uint viewID: SV_ViewID) {
  VSOutput o;

  uint particleIndex = vertexID / 6;
  uint cornerVertex = vertexID % 6;
  uint cornerIndex = kIndices[cornerVertex];

  Particle p = particles[particleIndex];

  float3 center = p.position;
  float size = p.size;

  float2 corner = kCorners[cornerIndex];
  float3 right = simParams.cameraRightWS[viewID].xyz;
  float3 up = simParams.cameraUpWS[viewID].xyz;

  float3 worldPos = center + right * (corner.x * size) + up * (corner.y * size);

  o.position = mul(simParams.viewProj[viewID], float4(worldPos, 1.0));
  o.localUV = corner;
  o.color = float4(p.color, 1.0);
  o.phase = p.phase;
  o.velocity = p.velocity;

  return o;
}

[shader("fragment")]
float4 fs_main(VSOutput i) : SV_Target {
  float2 uv = i.localUV;
  float r2 = dot(uv, uv);

  // cut square into disc
  if (r2 > 1.0)
    discard;

  // pretend it's a sphere cap
  float z = sqrt(max(0.0, 1.0 - r2));

  // soft center + rim
  float center = pow(z, 1.5);
  float rim = pow(1.0 - z, 2.5);

  // subtle pulse
  float pulse = 0.75 + 0.25 * sin(i.phase * 6.28318);

  // motion brightening
  float speed = length(i.velocity);
  float motionBoost = 1.0 + clamp(speed * 0.15, 0.0, 0.75);

  float3 base = i.color.rgb * i.color.a;
  float3 color = base * center * pulse * motionBoost;
  color += base * rim * 0.35;

  // soft alpha
  float alpha = smoothstep(1.0, 0.0, sqrt(r2));
  alpha *= 0.9;

  return float4(color, alpha);
}
