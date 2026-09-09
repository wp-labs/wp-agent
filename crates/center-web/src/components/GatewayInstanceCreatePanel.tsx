import { useEffect, useState, type FormEvent } from "react";
import { storeGatewayInitCurl } from "../api";
import { useCreateGatewayInstance } from "../hooks";
import {
  ErrorBanner,
  FormField,
  PrimaryButton,
  ReceiptCard,
  TextInput,
  formatDateTime,
  lifecycleLabel,
} from "./ui";
import styles from "./GatewayInstanceCreatePanel.module.css";

/** 安装指引代码块：展示并复制创建回执中的部署信息。 */
function CopyBlock({
  label,
  code,
  filename,
}: {
  label: string;
  code: string;
  filename?: string;
}) {
  const [copied, setCopied] = useState(false);

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1500);
    } catch {
      // 剪贴板不可用时保留可选中的原始文本，不阻断部署流程。
    }
  }

  function handleDownload() {
    if (!filename) return;
    const blobUrl = URL.createObjectURL(
      new Blob([code], { type: "text/plain" }),
    );
    const anchor = document.createElement("a");
    anchor.href = blobUrl;
    anchor.download = filename;
    anchor.click();
    URL.revokeObjectURL(blobUrl);
  }

  return (
    <div className={styles.step}>
      <div className={styles.stepLabel}>{label}</div>
      <div className={styles.codeBlock}>
        <div className={styles.codeActions}>
          {filename ? (
            <button
              type="button"
              className={styles.copyButton}
              onClick={handleDownload}
            >
              下载
            </button>
          ) : null}
          <button
            type="button"
            className={styles.copyButton}
            onClick={handleCopy}
          >
            {copied ? "已复制" : "复制"}
          </button>
        </div>
        <pre className={styles.codeText}>{code}</pre>
      </div>
    </div>
  );
}

/**
 * 提供可按需展开的实例创建流程。
 * 置备引导 Token 始终由 Center 签发，表单只收集实例标识和操作人信息。
 */
export function GatewayInstanceCreatePanel() {
  const [expanded, setExpanded] = useState(false);
  const mutation = useCreateGatewayInstance();

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    mutation.mutate({
      gatewayName: String(data.get("gatewayName") ?? ""),
      requestedBy: String(data.get("requestedBy") ?? ""),
    });
  }

  const created = mutation.data?.data;
  const instance = created?.instance;
  const install = created?.install;

  useEffect(() => {
    if (!instance || !install) return;
    // 创建回执中的凭证命令只写入当前会话，详情页可复制但实例列表不会重复暴露。
    storeGatewayInitCurl(instance.gatewayId, install.initCurl);
  }, [instance, install]);

  return (
    <section className={styles.panel}>
      <header className={styles.header}>
        <div className={styles.headerText}>
          <span className={styles.eyebrow}>Gateway Provisioning</span>
          <h2 className={styles.title}>新增网关实例</h2>
          <p className={styles.subtitle}>
            创建待接入实例并生成初始化 URL、镜像信息和一次性安装指引。
          </p>
        </div>
        <button
          type="button"
          className={styles.toggleButton}
          aria-expanded={expanded}
          aria-controls="gateway-instance-create-content"
          onClick={() => setExpanded((value) => !value)}
        >
          {expanded ? "收起创建" : "+ 创建实例"}
        </button>
      </header>

      {expanded ? (
        <div id="gateway-instance-create-content" className={styles.body}>
          <form className={styles.form} onSubmit={handleSubmit}>
            <div className={styles.formFields}>
              <FormField label="网关名称" hint="例如：gw-prod-east">
                <TextInput
                  name="gatewayName"
                  required
                  placeholder="请输入网关名称"
                />
              </FormField>
              <FormField label="申请者（requested_by）">
                <TextInput name="requestedBy" defaultValue="admin" required />
              </FormField>
            </div>
            <div className={styles.formAction}>
              <span className={styles.actionHint}>
                创建后实例进入“待部署”状态，置备引导 Token 由中心自动签发。
              </span>
              <PrimaryButton type="submit" disabled={mutation.isPending}>
                {mutation.isPending ? "创建中…" : "创建实例"}
              </PrimaryButton>
            </div>
          </form>

          {mutation.error ? (
            <ErrorBanner>创建失败：{String(mutation.error)}</ErrorBanner>
          ) : null}
          {instance ? (
            <ReceiptCard
              title="创建回执"
              fields={[
                ["网关 ID", instance.gatewayId],
                ["实例 ID", instance.instanceId],
                ["生命周期", lifecycleLabel(instance.lifecycleState)],
                ["创建时间", formatDateTime(instance.createdAt)],
              ]}
            />
          ) : null}
          {install ? (
            <div className={styles.install}>
              <div className={styles.installTitle}>安装指引</div>
              <CopyBlock
                label="① 一次性置备引导 Token（请立即妥善保存）"
                code={install.setupToken}
              />
              <CopyBlock
                label="② Docker 安装命令（已注入初始化 URL 与置备凭据）"
                code={install.installCommand}
              />
              <CopyBlock
                label="③ 云镜像地址（云服务器直接拉取）"
                code={install.cloudImage}
              />
              <CopyBlock
                label="④ 初始化 HTTPS URL（Gateway 启动后基于此 URL 初始化）"
                code={install.initUrl}
              />
              <CopyBlock
                label="⑤ curl 验证初始化 URL（Bearer 使用置备引导 Token）"
                code={install.initCurl}
              />
              {install.trustBundlePem ? (
                <CopyBlock
                  label="⑥ 控制中心 CA 证书（保存为 control-center.pem）"
                  code={install.trustBundlePem}
                  filename="control-center.pem"
                />
              ) : null}
              <p className={styles.installHint}>
                Gateway 启动后将携带置备引导 Token 访问初始化 URL，获取
                config.toml 和注册凭据，完成接入。
              </p>
            </div>
          ) : null}
        </div>
      ) : null}
    </section>
  );
}
