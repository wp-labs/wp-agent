import styles from "./SubsystemMacOSInstallCode.module.css";
import { ShellCode } from "./ShellCode";
import { CopyButton } from "./CopyButton";

interface SubsystemMacOSInstallCodeProps {
  command?: string;
  token?: string;
  loading?: boolean;
  children?: React.ReactNode;
}

export function SubsystemMacOSInstallCode({
  command,
  token,
  loading,
  children,
}: SubsystemMacOSInstallCodeProps) {
  const displayCommand = loading
    ? "安装代码加载中..."
    : (command ?? "暂无安装代码");
  const fullCommand =
    command && token
      ? `export WARP_INSIGHT_ENROLLMENT_TOKEN=${token}\n${command}`
      : command;

  return (
    <div className={styles.container}>
      <div className={styles.label}>macOS 安装命令</div>
      <code className={styles.code}>
        <ShellCode code={displayCommand} />
      </code>
      <div className={styles.actions}>
        <CopyButton text={command} label="复制命令" className={styles.copyButton} />
        <CopyButton
          text={fullCommand}
          label="复制完整命令（含 Token）"
          copiedLabel="已复制完整命令"
          className={styles.copyButton}
        />
      </div>
      {children}
    </div>
  );
}
