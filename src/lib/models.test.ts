import { describe, expect, it } from "vitest";
import {
  DEEPSEEK_MODELS,
  normalizeCloudModel,
  OPENAI_COMPAT_MODELS,
  pickDefaultCloudModel,
} from "./models";
import { presetForBaseUrl, CLOUD_API_PRESETS } from "./cloudApi";

describe("normalizeCloudModel", () => {
  it("keeps stored model when in available list", () => {
    expect(normalizeCloudModel("deepseek-reasoner", DEEPSEEK_MODELS)).toBe(
      "deepseek-reasoner"
    );
  });

  it("falls back to first available when unknown", () => {
    expect(normalizeCloudModel("unknown-model", DEEPSEEK_MODELS)).toBe(
      pickDefaultCloudModel(DEEPSEEK_MODELS)
    );
  });

  it("keeps custom model when allowCustom is true", () => {
    expect(
      normalizeCloudModel("my-local-model", OPENAI_COMPAT_MODELS, true)
    ).toBe("my-local-model");
  });
});

describe("cloudApi presets", () => {
  it("matches known provider base URLs", () => {
    expect(presetForBaseUrl("https://api.deepseek.com")?.id).toBe("deepseek");
    expect(presetForBaseUrl("https://api.openai.com/v1")?.id).toBe("openai");
  });

  it("returns null for unknown URLs", () => {
    expect(presetForBaseUrl("https://custom.example.com/v1")).toBeNull();
  });

  it("lists expected presets", () => {
    expect(CLOUD_API_PRESETS.map((p) => p.id)).toContain("groq");
    expect(CLOUD_API_PRESETS.map((p) => p.id)).toContain("lmstudio");
  });
});
