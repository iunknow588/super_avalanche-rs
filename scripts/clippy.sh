#!/usr/bin/env bash

# 开启调试（-x）、一旦发生错误立即退出（-e）、遇到未设置变量则报错（-u）。
# set -xue
set -xe

# 创建一个文件来跟踪我们修改的文件
echo "Creating tracking file for modified files..."
MODIFIED_FILES=$(mktemp)

# 修复自动生成文件中的重复属性
echo "Fixing duplicate attributes in auto-generated files..."
find . -name "*.rs" \( -path "**/pb/*.rs" -o -path "**/proto/pb/*.rs" -o -path "**/proto/tmp/*.rs" -o -path "**/generated/*.rs" -o -path "**/proto/generated/*.rs" \) | while read -r file; do
    # 检查文件是否同时具有外部和内部属性
    if grep -q "#\[allow(clippy::all)\]" "$file" && grep -q "#!\[allow(clippy::all)\]" "$file"; then
        # 删除内部属性
        sed -i '/#!\[allow(clippy::all)\]/d' "$file"
        # 将文件添加到修改文件列表
        echo "$file" >> "$MODIFIED_FILES"
    # 检查文件是否只有内部属性
    elif grep -q "#!\[allow(clippy::all)\]" "$file" && ! grep -q "#\[allow(clippy::all)\]" "$file"; then
        # 将内部属性替换为外部属性
        sed -i 's/#!\[allow(clippy::all)\]/#[allow(clippy::all)]/' "$file"
        # 将文件添加到修改文件列表
        echo "$file" >> "$MODIFIED_FILES"
    # 检查文件是否没有属性
    elif ! grep -q "#\[allow(clippy::all)\]" "$file" && ! grep -q "#!\[allow(clippy::all)\]" "$file"; then
        # 在文件顶部添加外部属性
        sed -i '1s/^/#[allow(clippy::all)]\n#[allow(clippy::wildcard_imports)]\n#[allow(clippy::missing_docs_in_private_items)]\n#[allow(clippy::missing_errors_doc)]\n#[allow(clippy::missing_panics_doc)]\n#[allow(clippy::missing_const_for_fn)]\n#[allow(clippy::default_trait_access)]\n#[allow(clippy::used_underscore_items)]\n/' "$file"
        # 将文件添加到修改文件列表
        echo "$file" >> "$MODIFIED_FILES"
    fi
done

# 使用自动生成文件的允许属性运行 clippy
echo "Running clippy..."

# 定义日志文件路径
CLIPPY_LOG_FILE="/home/lc/super_avalanche-rs/scripts/clippy.log"

# 确保日志文件目录存在
mkdir -p "$(dirname "$CLIPPY_LOG_FILE")"

# 清空之前的日志文件
> "$CLIPPY_LOG_FILE"

echo "Clippy output will be saved to $CLIPPY_LOG_FILE"

# 由于我们已经向自动生成的文件添加了 #[allow(clippy::all)]，
# 我们可以在所有文件上运行 clippy。自动生成的文件将被忽略
# 因为允许属性。
# 使用 || true 防止脚本在 clippy 发现问题时失败
RUSTFLAGS="--cfg=clippy" cargo clippy --all --all-features --tests --benches --examples -- \
-D clippy::suspicious \
-D clippy::style \
-D clippy::complexity \
-D clippy::perf \
-D clippy::pedantic \
-D clippy::nursery \
-D clippy::missing_docs_in_private_items \
-D clippy::missing_errors_doc \
-D clippy::missing_panics_doc \
-D warnings \
-D clippy::large_stack_arrays \
-D clippy::missing_safety_doc \
-D clippy::redundant_pub_crate \
-D clippy::unwrap_in_result \
-D clippy::wildcard_dependencies \
-A clippy::multiple_crate_versions \
-A clippy::wildcard_imports \
-A clippy::missing_docs_in_private_items \
-A clippy::missing_errors_doc \
-A clippy::missing_panics_doc \
-A clippy::missing_const_for_fn \
-A clippy::default_trait_access \
-A clippy::used_underscore_items \
-A clippy::doc_markdown \
-A clippy::too_long_first_doc_paragraph \
-A clippy::must_use_candidate \
-A clippy::use_self \
-A clippy::borrow_as_ptr 2>&1 | tee "$CLIPPY_LOG_FILE" || true

# 将自动生成的文件恢复到原始状态
echo "Restoring auto-generated files..."
if [ -f "$MODIFIED_FILES" ]; then
    while read -r file; do
        if [ -f "$file" ]; then
            # 从文件顶部删除允许属性
            sed -i '1d' "$file"
        fi
    done < "$MODIFIED_FILES"
    rm -f "$MODIFIED_FILES"
fi

echo "Clippy check completed!"
