import { useTranslation } from "react-i18next"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Button } from "@/components/ui/button"

interface FreeApiProvider {
  name: string
  zh: string
  en: string
  url: string
}

const PROVIDERS: FreeApiProvider[] = [
  {
    name: "OpenRouter",
    zh: "OpenRouter 是一个统一 API 平台，免费模型限制为 20 次请求/分钟、50 次/天（充值 $10 后提升至 1000 次/天），所有免费模型共享配额。免费可用的模型包括 DeepSeek R1、Gemma 3 27B/12B/4B/3n、Llama 3.3 70B/3.2 3B、Hermes 3 405B、Mistral Small 3.1 24B、Trinity Large/Mini、GPT-OSS 120B/20B、Qwen3 4B/Coder/Next 80B、Nemotron 系列、GLM 4.5 Air、Step 3.5 Flash、Dolphin Mistral 24B、Liquid LFM 2.5 等二十余款。",
    en: "OpenRouter is a unified API gateway. Its free tier is limited to 20 requests per minute and 50 requests per day, raised to 1,000 per day after topping up $10. All free models share the same quota. Free models include DeepSeek R1, Gemma 3 27B/12B/4B/3n, Llama 3.3 70B and 3.2 3B, Hermes 3 405B, Mistral Small 3.1 24B, Trinity Large and Mini, GPT-OSS 120B/20B, Qwen3 4B/Coder/Next 80B, the Nemotron family, GLM 4.5 Air, Step 3.5 Flash, Dolphin Mistral 24B, Liquid LFM 2.5, and more than twenty other models.",
    url: "https://openrouter.ai",
  },
  {
    name: "Google AI Studio",
    zh: "Google AI Studio 提供 Gemini 和 Gemma 系列的免费套餐。其中 Gemini 3 Flash 为 20 次/天，Gemini 3.1 Flash-Lite 为 500 次/天，Gemini 2.5 Flash 和 Gemini 2.5 Flash-Lite 各 20 次/天，Gemma 3 27B/12B/4B/1B 均为 14,400 次/天。",
    en: "Google AI Studio offers free access to Gemini and Gemma models. Gemini 3 Flash is limited to 20 requests per day, Gemini 3.1 Flash-Lite to 500 per day, Gemini 2.5 Flash and Gemini 2.5 Flash-Lite to 20 per day each, and Gemma 3 27B/12B/4B/1B to 14,400 requests per day.",
    url: "https://aistudio.google.com",
  },
  {
    name: "Mistral La Plateforme",
    zh: "Mistral La Plateforme 提供免费层（Experiment 计划），需同意数据训练并完成手机验证。限制为 1 次/秒、50 万 token/分钟、10 亿 token/月，包含 Mistral 全系开源和闭源模型。",
    en: "Mistral La Plateforme offers a free Experiment plan. You must allow training data usage and complete phone verification. Limits are 1 request per second, 500,000 tokens per minute, and 1 billion tokens per month, covering both open and closed Mistral models.",
    url: "https://console.mistral.ai/",
  },
  {
    name: "Mistral Codestral",
    zh: "Mistral Codestral 目前免费使用，需手机验证，限制为 30 次/分钟、2000 次/天。",
    en: "Mistral Codestral is currently free to use with phone verification, limited to 30 requests per minute and 2,000 requests per day.",
    url: "https://codestral.mistral.ai/",
  },
  {
    name: "NVIDIA NIM",
    zh: "NVIDIA NIM 需手机验证，限制为 40 次/分钟，提供多种开源模型。",
    en: "NVIDIA NIM requires phone verification, is limited to 40 requests per minute, and provides multiple open-weight models.",
    url: "https://build.nvidia.com/explore/discover",
  },
  {
    name: "Cerebras",
    zh: "Cerebras 免费提供 GPT-OSS 120B 和 Llama 3.1 8B，均为 14,400 次/天、30 次/分钟。",
    en: "Cerebras offers GPT-OSS 120B and Llama 3.1 8B for free, both with limits of 14,400 requests per day and 30 requests per minute.",
    url: "https://cloud.cerebras.ai/",
  },
  {
    name: "Groq",
    zh: "Groq 免费提供多款模型，其中 Llama 3.1 8B 为 14,400 次/天，Llama 3.3 70B、Llama 4 Maverick/Scout、Qwen3 32B、GPT-OSS 120B/20B、Kimi K2 等均为 1,000 次/天。",
    en: "Groq offers multiple free models. Llama 3.1 8B is capped at 14,400 requests per day, while Llama 3.3 70B, Llama 4 Maverick and Scout, Qwen3 32B, GPT-OSS 120B/20B, Kimi K2, and others are capped at 1,000 requests per day.",
    url: "https://console.groq.com",
  },
  {
    name: "Cohere",
    zh: "Cohere 限制为 20 次/分钟、1,000 次/月，所有模型共享配额。可用模型包括 Command A 系列、Command R/R+ 系列、Aya Expanse/Vision 32B、Tiny Aya 系列等十余款。",
    en: "Cohere is limited to 20 requests per minute and 1,000 requests per month, shared across all models. Available models include the Command A family, Command R/R+, Aya Expanse/Vision 32B, Tiny Aya, and more than ten others.",
    url: "https://cohere.com",
  },
  {
    name: "GitHub Models",
    zh: "GitHub Models 的额度取决于 Copilot 订阅级别，输入输出 token 限制较严格。可用模型包括 GPT-5/4.1/o3/o4-mini、DeepSeek R1/V3、Llama 全系列、Grok 3、Mistral、Phi-4 等数十款。",
    en: "GitHub Models quota depends on your Copilot subscription tier, and token limits are relatively strict. Available models include GPT-5/4.1/o3/o4-mini, DeepSeek R1/V3, the Llama family, Grok 3, Mistral, Phi-4, and dozens more.",
    url: "https://github.com/marketplace/models",
  },
  {
    name: "Cloudflare Workers AI",
    zh: "Cloudflare Workers AI 每天提供 10,000 neurons 免费额度。可用模型包括 Llama 3.3 70B/4 Scout、Gemma 3 12B、Mistral Small 3.1、GPT-OSS 120B/20B、GLM 4.7 Flash、Qwen 2.5 Coder 32B、DeepSeek R1 Distill 等数十款。",
    en: "Cloudflare Workers AI includes 10,000 free neurons per day. Available models include Llama 3.3 70B and 4 Scout, Gemma 3 12B, Mistral Small 3.1, GPT-OSS 120B/20B, GLM 4.7 Flash, Qwen 2.5 Coder 32B, DeepSeek R1 Distill, and many more.",
    url: "https://developers.cloudflare.com/workers-ai",
  },
  {
    name: "HuggingFace Inference Providers",
    zh: "HuggingFace Inference Providers 提供每月 $0.10 的免费额度，Serverless 推理限于 10GB 以下模型。",
    en: "HuggingFace Inference Providers offers $0.10 of free credit per month. Serverless inference is limited to models smaller than 10GB.",
    url: "https://huggingface.co/docs/inference-providers/en/index",
  },
  {
    name: "Vercel AI Gateway",
    zh: "Vercel AI Gateway 提供每月 $5 的免费额度，可路由至多家提供商。",
    en: "Vercel AI Gateway includes $5 of free credit per month and can route requests to multiple providers.",
    url: "https://vercel.com/docs/ai-gateway",
  },
  {
    name: "OpenCode Zen",
    zh: "OpenCode Zen 是一个 AI 网关，免费模型包括 Big Pickle Stealth、MiniMax M2.5 Free、Arcee Large Preview Free。",
    en: "OpenCode Zen is an AI gateway. Its free models include Big Pickle Stealth, MiniMax M2.5 Free, and Arcee Large Preview Free.",
    url: "https://opencode.ai/docs/zen/",
  },
  {
    name: "Poixe AI",
    zh: "Poixe AI 聚合了多种免费模型，需要 API key。",
    en: "Poixe AI aggregates multiple free models and requires its own API key.",
    url: "https://poixe.com/",
  },
  {
    name: "硅基流动 SiliconFlow",
    zh: "硅基流动 提供免费的 DeepSeek R1 蒸馏版，另有部分国产模型及文生图模型限时免费。",
    en: "SiliconFlow provides a free distilled DeepSeek R1 model, and some Chinese language models plus text-to-image models are available for free for limited periods.",
    url: "https://cloud.siliconflow.cn/models",
  },
  {
    name: "无问芯穹 Infini-AI",
    zh: "无问芯穹 提供免费的 DeepSeek R1。",
    en: "Infini-AI offers free access to DeepSeek R1.",
    url: "https://cloud.infini-ai.com/promotion",
  },
  {
    name: "智谱清言 BigModel",
    zh: "智谱清言 的 GLM-4-Flash 永久免费，支持函数调用。",
    en: "BigModel offers GLM-4-Flash permanently for free and supports function calling.",
    url: "https://open.bigmodel.cn/console/overview",
  },
  {
    name: "glhf.chat",
    zh: "glhf.chat 提供多种开源旗舰模型的免费 API，也支持自行部署模型。",
    en: "glhf.chat provides free APIs for multiple flagship open-weight models and also supports self-hosted models.",
    url: "https://glhf.chat/landing/home",
  },
  {
    name: "Fireworks",
    zh: "Fireworks 提供 $1 试用额度，支持多种开源模型。",
    en: "Fireworks offers $1 in trial credit and supports many open-weight models.",
    url: "https://fireworks.ai/",
  },
  {
    name: "Baseten",
    zh: "Baseten 提供 $30 试用额度，按计算时间计费，支持多种模型。",
    en: "Baseten offers $30 in trial credit, bills by compute time, and supports many models.",
    url: "https://app.baseten.co/",
  },
  {
    name: "Nebius",
    zh: "Nebius 提供 $1 试用额度，支持多种开源模型。",
    en: "Nebius offers $1 in trial credit and supports many open-weight models.",
    url: "https://tokenfactory.nebius.com/",
  },
  {
    name: "Novita",
    zh: "Novita 提供 $0.5 试用额度（1 年有效），支持多种开源模型。",
    en: "Novita offers $0.5 in trial credit valid for one year and supports many open-weight models.",
    url: "https://novita.ai/",
  },
  {
    name: "AI21",
    zh: "AI21 提供 $10 试用额度（3 个月有效），可用模型为 Jamba 系列。",
    en: "AI21 offers $10 in trial credit valid for three months, mainly for the Jamba family.",
    url: "https://studio.ai21.com/",
  },
  {
    name: "Upstage",
    zh: "Upstage 提供 $10 试用额度（3 个月有效），可用模型为 Solar Pro/Mini。",
    en: "Upstage offers $10 in trial credit valid for three months, covering Solar Pro and Solar Mini.",
    url: "https://console.upstage.ai/",
  },
  {
    name: "NLP Cloud",
    zh: "NLP Cloud 提供 $15 试用额度，需手机验证，支持多种开源模型。",
    en: "NLP Cloud offers $15 in trial credit, requires phone verification, and supports many open-weight models.",
    url: "https://nlpcloud.com/home",
  },
  {
    name: "阿里云国际版 Model Studio",
    zh: "阿里云国际版 Model Studio 为每个模型提供 100 万免费 token，可用模型为 Qwen 系列开源和闭源版本。",
    en: "Alibaba Cloud International Model Studio offers 1 million free tokens per model, including both open and closed Qwen variants.",
    url: "https://bailian.console.alibabacloud.com/",
  },
  {
    name: "Modal",
    zh: "Modal 注册即享 $5/月额度，添加支付方式后提升至 $30/月，按计算时间计费。",
    en: "Modal gives you $5 per month on signup, increased to $30 per month after adding a payment method, with billing based on compute time.",
    url: "https://modal.com",
  },
  {
    name: "Inference.net",
    zh: "Inference.net 提供 $1 试用额度，回复邮件调查可再获 $25，支持多种开源模型。",
    en: "Inference.net offers $1 in trial credit, plus another $25 after replying to its survey email, and supports many open-weight models.",
    url: "https://inference.net",
  },
  {
    name: "Hyperbolic",
    zh: "Hyperbolic 提供 $1 试用额度。可用模型包括 DeepSeek V3/R1、Llama 3.1/3.2/3.3 系列、Qwen 2.5/3 系列、GPT-OSS 等。",
    en: "Hyperbolic offers $1 in trial credit. Available models include DeepSeek V3/R1, Llama 3.1/3.2/3.3, Qwen 2.5/3, GPT-OSS, and more.",
    url: "https://app.hyperbolic.ai/",
  },
  {
    name: "SambaNova Cloud",
    zh: "SambaNova Cloud 提供 $5 试用额度（3 个月有效）。可用模型包括 Llama 3.1/3.3/4、Qwen3、DeepSeek R1/V3 系列、MiniMax M2.5 等。",
    en: "SambaNova Cloud offers $5 in trial credit valid for three months. Available models include Llama 3.1/3.3/4, Qwen3, DeepSeek R1/V3, MiniMax M2.5, and more.",
    url: "https://cloud.sambanova.ai/",
  },
  {
    name: "Scaleway",
    zh: "Scaleway 提供 100 万免费 token。可用模型包括 DeepSeek R1 Distill 70B、Gemma 3 27B、Llama 3.3 70B、Mistral 系列、GPT-OSS 120B、Qwen3 系列等。",
    en: "Scaleway offers 1 million free tokens. Available models include DeepSeek R1 Distill 70B, Gemma 3 27B, Llama 3.3 70B, the Mistral family, GPT-OSS 120B, the Qwen3 family, and more.",
    url: "https://console.scaleway.com/generative-api/models",
  },
]

export function FreeApiHelpDialog({
  open,
  onOpenChange,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  const { t, i18n } = useTranslation()
  const isZh = i18n.language === "zh-CN"

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="overflow-hidden p-0 md:max-h-[500px] md:max-w-[700px] lg:max-w-[800px]">
        <div className="flex h-[500px] flex-col">
          <DialogHeader className="shrink-0 px-6 pt-6">
            <DialogTitle>{t("page.dashboard.credentialsDialog.title")}</DialogTitle>
            <DialogDescription>
              {t("page.dashboard.credentialsDialog.description")}
            </DialogDescription>
          </DialogHeader>

          <div className="flex-1 overflow-y-auto px-6 py-4">
            <div className="flex flex-col gap-3">
              {PROVIDERS.map((provider) => (
                <section key={provider.name} className="rounded-lg border bg-muted/20 p-4">
                  <h3 className="font-medium">{provider.name}</h3>
                  <div className="mt-2 space-y-2 text-sm leading-6 text-muted-foreground">
                    <p>{isZh ? provider.zh : provider.en}</p>
                    <p>
                      <a
                        href={provider.url}
                        target="_blank"
                        rel="noreferrer"
                        className="text-primary"
                      >
                        {provider.url}
                      </a>
                    </p>
                  </div>
                </section>
              ))}
            </div>
          </div>

          <DialogFooter className="mx-0 mb-0 shrink-0">
            <Button variant="outline" size="lg" onClick={() => onOpenChange(false)}>
              {t("page.dashboard.credentialsDialog.close")}
            </Button>
          </DialogFooter>
        </div>
      </DialogContent>
    </Dialog>
  )
}
