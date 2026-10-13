<p align="center">
  <p align="center">
   <img width="150" height="150" src="./apps/desktop/src-tauri/icons/Square310x310Logo.png" alt="Logo">
  </p>
	<h1 align="center"><b>CapForZhaDao（Cap 快速录制增强版）</b></h1>
	<p align="center">
		开源屏幕录制工具，基于 Cap 中文版 <code>0.4.3-cn</code> 做小幅增强（v0.5.0-opt）
    <br />
    <a href="https://github.com/zhadao/capTestforZha"><strong>GitHub 仓库 »</strong></a>
    <br />
    <br />
    <b>支持平台：</b>
			Windows
    <br />
  </p>
</p>
<br/>

## 📖 项目简介

CapForZhaDao 是基于 <a href="https://github.com/lid664951-crypto/Cap">Cap 中文版</a>（同步 <a href="https://github.com/CapSoftware/Cap">Cap 官方</a> 0.4.3）改造的开源屏幕录制工具。改造范围刻意保持极小：**不改动任何录制/取屏核心管线**，只围绕「快速录制」做体验增强，并清理原作者的推广信息。

## ✨ 本版增强（v0.5.0-opt）

- **即时模式 GIF/MP4 格式选择**：录制前一键选择「GIF图」或「MP4」，选择持久化；GIF 采用录后转码（ffmpeg 解码 + gifski 编码，12fps / 质量 90 / 无限循环，单段上限 60 秒）
- **录完询问复制**：录制完成弹出「是否复制到剪贴板？」询问条，点击才以文件形式（CF_HDROP）复制，可直接粘贴到聊天窗口、文档等任意位置
- **只读预览工作区**：类工作室布局的即时预览窗口，GIF/MP4 自适应渲染 + 复制/导出/打开文件夹，无任何剪辑功能
- **成品落盘到你自己的目录**：默认项目 `data` 目录，按 `年/月/日` 分层，命名 `cap_日期_时间.格式`；设置中可自定义目录与保留条数（默认 50，超出自动清理）
- **去原作者推广信息**：删除「赞助我们」「反馈」板块及收款码、微信二维码、B站账号等个人联系方式与外链

## ✨ 继承自 Cap 中文版的特性

- **完全汉化**：界面和功能说明已全部汉化，提供更友好的中文用户体验
- **移除登录限制**：无需登录即可使用所有功能
- **去除付费板块**：所有功能完全免费使用

## 📸 界面预览

### 首页
<img src="./UI界面/首页.png" alt="首页" width="800" />

### 编辑器页面
<img src="./UI界面/编辑器页面.png" alt="编辑器页面" width="800" />

### 设置页面
<img src="./UI界面/设置页面.png" alt="设置页面" width="800" />

## 🚀 安装使用

### 方法一：直接运行
1. 从 GitHub 仓库下载最新版本
2. 解压后运行相应平台的可执行文件

### 方法二：源码构建
```bash
# 克隆仓库
git clone https://github.com/zhadao/capTestforZha.git
cd capTestforZha

# 安装依赖
pnpm install

# 构建桌面应用
pnpm build:desktop

# 运行开发版本
pnpm dev:desktop
```

## 🛠️ 技术栈

- **前端**：React (Next.js)、TypeScript、SolidStart、TailwindCSS
- **后端**：Rust、Tauri、Drizzle (ORM)、MySQL
- **构建工具**：Turborepo

## 📄 许可证

本项目基于原项目的许可证进行分发：

- `cap-camera*` 和 `scap-*` 系列代码使用 MIT 许可证
- 第三方组件使用其原始许可证
- 其他内容使用 AGPLv3 许可证

详细信息请查看 [LICENSE](LICENSE) 文件。

## 🔗 相关链接

- **Cap 官方项目**：<a href="https://github.com/CapSoftware/Cap">https://github.com/CapSoftware/Cap</a>
- **Cap 中文版（上游）**：<a href="https://github.com/lid664951-crypto/Cap">https://github.com/lid664951-crypto/Cap</a>
- **本项目**：<a href="https://github.com/zhadao/capTestforZha">https://github.com/zhadao/capTestforZha</a>

## 🤝 贡献

欢迎提交 Issue 和 Pull Request 来帮助改进这个项目！

## 📞 联系方式

如有问题或建议，请在 GitHub 仓库中提交 Issue。
