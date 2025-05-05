# scripts 目录说明

`scripts`目录下共包含以下脚本文件：

- build.release.sh
- check_crate_version.sh
- tests.avalanchego-byzantine.sh
- tests.avalanchego-conformance.sh
- tests.avalanchego-e2e.sh
- tests.fuzz.sh
- tests.lint.sh
- tests.unit.sh
- tests.unused.sh

## 各脚本功能简介

**build.release.sh**  
用于构建项目的发布版本。通常会执行编译、打包等操作，生成可用于生产环境的二进制文件。

**check_crate_version.sh**  
检查 Rust crate 的版本信息。该脚本可能会比对本地 crate 版本和远程仓库的版本，确保依赖项是最新或符合要求的版本。

**tests.avalanchego-byzantine.sh**  
用于运行与 Byzantine（拜占庭）相关的测试，主要测试系统在存在恶意节点或异常情况下的容错能力。

**tests.avalanchego-conformance.sh**  
运行一致性测试（conformance test），确保实现符合协议规范或标准。

**tests.avalanchego-e2e.sh**  
运行端到端（E2E）测试，测试整个系统或主要流程的完整性和正确性。

**tests.fuzz.sh**  
运行模糊测试（fuzz test），通过随机或异常输入测试系统的健壮性和安全性。

**tests.lint.sh**  
运行代码风格检查（lint），确保代码符合规范，避免常见的编程错误或不规范写法。

**tests.unit.sh**  
运行单元测试，验证各个模块或函数的正确性。

**tests.unused.sh**  
检查代码中未被使用的部分，帮助清理无用代码，提高代码质量。

---
如果你需要详细分析某一个脚本的具体内容和实现逻辑，可以告诉我具体的脚本名称，我会进一步为你解读。
