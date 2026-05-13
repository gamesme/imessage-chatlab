# imessage-chatlab

> [English](./README.md) | 简体中文

将本机 iMessage 数据导出为 [ChatLab](https://chatlab.fun) v0.0.2 标准 JSON 格式。
每个会话一个 JSON 文件,可选附件复制,可选将头像以 base64 Data URL 内嵌进 JSON。

> 基于 [@ReagentX](https://github.com/ReagentX) 的 [`imessage-database`](https://crates.io/crates/imessage-database) 构建。
> 完整署名信息见 `NOTICE.md`。

## 安装

```bash
cargo install imessage-chatlab
```

或从源码安装:

```bash
cargo install --git https://github.com/gamesme/imessage-chatlab
```

## 用法

把本机 iMessage 数据库导出为 ChatLab JSON,同时复制所有附件并把头像内嵌为 Data URL:

```bash
imessage-chatlab -c clone -o ~/imessage_chatlab_export
```

更轻量的导出:不复制附件,不内嵌头像:

```bash
imessage-chatlab --embed-avatars=false -o ~/imessage_chatlab_export
```

### 命令行选项

```text
-c, --copy-method <clone|basic|full|disabled>
        附件处理方式
        `disabled`(默认):不复制附件
        `clone`:原样复制附件
        `basic`:复制并将 HEIC 图片转为 JPEG
        `full`:再额外把 CAF/MOV 音视频转为 MP4
-p, --db-path <path>
        自定义 iMessage 数据库路径(可选)
        macOS:chat.db 文件
        iOS:iOS 备份根目录
-r, --attachment-root <path>
        自定义附件数据查找目录(可选)
-a, --platform <macOS|iOS>
        来源平台,省略时自动检测
-o, --export-path <path>
        输出目录(默认:~/imessage_chatlab_export)
-s, --start-date <YYYY-MM-DD>
        起始日期(包含本日)
-e, --end-date <YYYY-MM-DD>
        结束日期(不包含本日)
-m, --custom-name <name>
        在导出中为数据库所有者使用的自定义名称
-i, --use-caller-id
        在导出中用所有者的 caller ID 代替 "Me"
-t, --conversation-filter <filter>
        按参与者(姓名、号码、邮箱)过滤会话
-x, --cleartext-password <password>
        加密 iOS 备份的密码
-n, --contacts-path <path>
        自定义 AddressBook / 通讯录数据库路径(可选)
    --embed-avatars <true|false>
        是否将联系人 / 群头像以 base64 Data URL 内嵌(默认:true)
-h, --help
    --version
```

## 输出格式

完整的输出格式说明见 [ChatLab 标准格式规范](https://chatlab.fun/cn/standard/chatlab-format.html)。
消息 `type` 字段遵循 ChatLab 枚举(0 文本、1 图片、2 语音、3 视频、4 文件、
5 表情贴纸、7 链接、23 通话、80 系统、81 撤回、99 其他)。媒体消息的 `content`
字段采用带标签的占位符:

| 场景 | `content` |
|---|---|
| 已开启附件复制的图片 | `[Image] attachments/12/8421.jpeg` |
| 带文字的图片 | `[Image] attachments/12/8421.jpeg — look at this` |
| 带转写的语音 | `[Voice] msg.caf — Transcription: hello` |

## 已知限制

- 较新版本 macOS 的通讯录可能把头像存在 `ZABCDLIKENESS`(而不是 `ZABCDIMAGE`),
  或以外部 blob 形式放在 `.AddressBook-v22_SUPPORT/_EXTERNAL_DATA/`。
  这些情况下头像会静默为空。
- iOS 上 `ABMultiValue.property` 中电话 / 邮箱的属性编号为尽力解析。
- 群头像通过 `fs::read` 读取,不走加密 iOS 备份的解密路径;遇到加密备份时
  `meta.groupAvatar` 会被静默省略。
- 共享位置开始 / 结束事件目前会落到 `type: 0`、`content: null`。
- 当请求复制附件但源文件无法读取 / 解密 / 复制时,JSON 仍会引用原始文件名,
  没有带内失败标识。

## 许可证

GPL-3.0-or-later。详见 `LICENSE` 与 `NOTICE.md`。
