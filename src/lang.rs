use clap::ValueEnum;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Language {
    En,
    Zh,
}

#[allow(dead_code)]
impl Language {
    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Zh => "zh",
        }
    }

    pub fn menu_title(self) -> &'static str {
        match self {
            Self::En => "wetwin",
            Self::Zh => "wetwin 微信多开管理",
        }
    }

    pub fn menu_items(self) -> [&'static str; 7] {
        match self {
            Self::En => [
                "1. List installed WeChat apps",
                "2. Create a WeChat copy",
                "3. Open a specific copy",
                "4. Open all WeChat apps",
                "5. Remove a WeChat copy",
                "6. Run doctor",
                "0. Exit",
            ],
            Self::Zh => [
                "1. 查看已安装的微信应用",
                "2. 创建微信副本",
                "3. 打开指定副本",
                "4. 打开全部微信应用",
                "5. 删除微信副本",
                "6. 运行环境检查",
                "0. 退出",
            ],
        }
    }

    pub fn select_action(self) -> &'static str {
        match self {
            Self::En => "Select an action",
            Self::Zh => "请选择操作",
        }
    }

    pub fn create_index(self) -> &'static str {
        match self {
            Self::En => "Enter the copy index to create",
            Self::Zh => "请输入要创建的副本编号",
        }
    }

    pub fn create_force(self) -> &'static str {
        match self {
            Self::En => "If the target already exists, recreate it with --force?",
            Self::Zh => "如果目标已存在，是否使用 --force 重新创建？",
        }
    }

    pub fn open_target(self) -> &'static str {
        match self {
            Self::En => "Enter the copy index to open, or type 'all'",
            Self::Zh => "请输入要打开的副本编号，或输入 'all'",
        }
    }

    pub fn remove_index(self) -> &'static str {
        match self {
            Self::En => "Enter the copy index to remove",
            Self::Zh => "请输入要删除的副本编号",
        }
    }

    pub fn remove_skip_confirm(self) -> &'static str {
        match self {
            Self::En => "Skip the second confirmation prompt?",
            Self::Zh => "是否跳过二次确认？",
        }
    }

    pub fn bye(self) -> &'static str {
        match self {
            Self::En => "Bye.",
            Self::Zh => "已退出。",
        }
    }

    pub fn invalid_choice(self) -> &'static str {
        match self {
            Self::En => "Invalid choice. Please enter one of the menu numbers.",
            Self::Zh => "无效选择，请输入菜单中的编号。",
        }
    }

    pub fn yes_no_suffix(self) -> &'static str {
        match self {
            Self::En => "[y/N]",
            Self::Zh => "[y/N]",
        }
    }

    pub fn invalid_index(self, value: &str) -> String {
        match self {
            Self::En => format!("Invalid index '{value}'. Please enter a number such as 2."),
            Self::Zh => format!("无效编号 '{value}'，请输入类似 2 这样的数字。"),
        }
    }

    pub fn invalid_open_target(self, value: &str) -> String {
        match self {
            Self::En => {
                format!("Invalid target '{value}'. Please enter 2, 3, 4, or type all.")
            }
            Self::Zh => format!("无效目标 '{value}'，请输入 2、3、4 这样的编号，或输入 all。"),
        }
    }

    pub fn index_too_small(self) -> &'static str {
        match self {
            Self::En => "Index must be 2 or greater.",
            Self::Zh => "编号必须大于等于 2。",
        }
    }

    pub fn index_example(self) -> &'static str {
        match self {
            Self::En => "Example: 2",
            Self::Zh => "例如：2",
        }
    }

    pub fn open_target_example(self) -> &'static str {
        match self {
            Self::En => "Example: 2 or all",
            Self::Zh => "例如：2 或 all",
        }
    }

    pub fn choose_language(self) -> &'static str {
        match self {
            Self::En => "Select language / 选择语言",
            Self::Zh => "选择语言 / Select language",
        }
    }

    pub fn language_options(self) -> [&'static str; 3] {
        match self {
            Self::En | Self::Zh => ["1. 中文", "2. English", "Press Enter to use the default."],
        }
    }

    pub fn only_supports_macos(self) -> &'static str {
        match self {
            Self::En => "wetwin only supports macOS.",
            Self::Zh => "wetwin 仅支持 macOS。",
        }
    }

    pub fn wechat_installation(self) -> &'static str {
        match self {
            Self::En => "WeChat installation",
            Self::Zh => "微信安装情况",
        }
    }

    pub fn original_label(self) -> &'static str {
        match self {
            Self::En => "Original:",
            Self::Zh => "原版应用：",
        }
    }

    pub fn found(self) -> &'static str {
        match self {
            Self::En => "FOUND",
            Self::Zh => "已找到",
        }
    }

    pub fn missing(self) -> &'static str {
        match self {
            Self::En => "MISSING",
            Self::Zh => "缺失",
        }
    }

    pub fn bundle_id_label(self) -> &'static str {
        match self {
            Self::En => "Bundle ID",
            Self::Zh => "Bundle ID",
        }
    }

    pub fn unavailable(self) -> &'static str {
        match self {
            Self::En => "Unavailable",
            Self::Zh => "无法读取",
        }
    }

    pub fn local_copies(self) -> &'static str {
        match self {
            Self::En => "Local copies",
            Self::Zh => "本地副本",
        }
    }

    pub fn no_local_copies(self) -> &'static str {
        match self {
            Self::En => "No WeChat copies were found in /Applications.",
            Self::Zh => "在 /Applications 中未发现微信副本。",
        }
    }

    pub fn source_not_found(self) -> &'static str {
        match self {
            Self::En => "Source app not found: /Applications/WeChat.app",
            Self::Zh => "未找到源应用：/Applications/WeChat.app",
        }
    }

    pub fn target_exists(self, path: &str) -> String {
        match self {
            Self::En => {
                format!("Target already exists: {path}. Please remove it first or use --force.")
            }
            Self::Zh => format!("目标已存在：{path}。请先删除它，或使用 --force。"),
        }
    }

    pub fn removing_existing_copy(self) -> &'static str {
        match self {
            Self::En => "Removing existing copy because --force was provided:",
            Self::Zh => "检测到 --force，正在先删除已有副本：",
        }
    }

    pub fn copying(self) -> &'static str {
        match self {
            Self::En => "Copying",
            Self::Zh => "正在复制",
        }
    }

    pub fn info_plist_missing(self, path: &str) -> String {
        match self {
            Self::En => format!("Info.plist not found after copy: {path}"),
            Self::Zh => format!("复制完成后未找到 Info.plist：{path}"),
        }
    }

    pub fn updating_bundle_id(self, bundle_id: &str) -> String {
        match self {
            Self::En => format!("Updating Bundle Identifier to {bundle_id}"),
            Self::Zh => format!("正在更新 Bundle ID 为 {bundle_id}"),
        }
    }

    pub fn copying_preferences(self) -> &'static str {
        match self {
            Self::En => "Copying original WeChat preferences",
            Self::Zh => "正在复制原版微信偏好设置",
        }
    }

    pub fn preferences_copied(self) -> &'static str {
        match self {
            Self::En => "Preferences copied from the original WeChat app.",
            Self::Zh => "已复制原版微信的偏好设置。",
        }
    }

    pub fn preferences_not_found(self) -> &'static str {
        match self {
            Self::En => "Original WeChat preferences were not found, so the new copy will use its own defaults.",
            Self::Zh => "未找到原版微信偏好设置，新副本将使用自己的默认配置。",
        }
    }

    pub fn preferences_copy_warning(self, details: &str) -> String {
        match self {
            Self::En => format!("Could not copy original WeChat preferences: {details}"),
            Self::Zh => format!("无法复制原版微信偏好设置：{details}"),
        }
    }

    pub fn preferences_export_failed(self) -> &'static str {
        match self {
            Self::En => "Failed to export preferences from com.tencent.xinWeChat.",
            Self::Zh => "无法从 com.tencent.xinWeChat 导出偏好设置。",
        }
    }

    pub fn preferences_import_failed(self, bundle_id: &str) -> String {
        match self {
            Self::En => format!("Failed to import preferences into {bundle_id}."),
            Self::Zh => format!("无法将偏好设置导入到 {bundle_id}。"),
        }
    }

    pub fn applying_language_preference(self) -> &'static str {
        match self {
            Self::En => "Applying explicit language preference",
            Self::Zh => "正在写入显式语言偏好",
        }
    }

    pub fn language_preference_applied(self) -> &'static str {
        match self {
            Self::En => "Language preference applied.",
            Self::Zh => "语言偏好已写入。",
        }
    }

    pub fn language_write_failed(self, bundle_id: &str) -> String {
        match self {
            Self::En => format!("Failed to write AppleLanguages for {bundle_id}."),
            Self::Zh => format!("无法为 {bundle_id} 写入 AppleLanguages。"),
        }
    }

    pub fn locale_write_failed(self, bundle_id: &str) -> String {
        match self {
            Self::En => format!("Failed to write AppleLocale for {bundle_id}."),
            Self::Zh => format!("无法为 {bundle_id} 写入 AppleLocale。"),
        }
    }

    pub fn clearing_quarantine(self) -> &'static str {
        match self {
            Self::En => "Clearing quarantine attributes",
            Self::Zh => "正在清除 quarantine 属性",
        }
    }

    pub fn signing(self) -> &'static str {
        match self {
            Self::En => "Applying local ad-hoc code signature",
            Self::Zh => "正在进行本地 ad-hoc 签名",
        }
    }

    pub fn created_successfully(self) -> &'static str {
        match self {
            Self::En => "Created successfully:",
            Self::Zh => "创建成功：",
        }
    }

    pub fn app_copy_not_found(self, path: &str) -> String {
        match self {
            Self::En => format!("App copy not found: {path}"),
            Self::Zh => format!("未找到应用副本：{path}"),
        }
    }

    pub fn opened(self) -> &'static str {
        match self {
            Self::En => "Opened",
            Self::Zh => "已打开",
        }
    }

    pub fn original_warning_not_found(self) -> &'static str {
        match self {
            Self::En => "Warning: original app not found:",
            Self::Zh => "警告：未找到原版应用：",
        }
    }

    pub fn no_wechat_apps_to_open(self) -> &'static str {
        match self {
            Self::En => "No WeChat apps were found to open.",
            Self::Zh => "没有可打开的微信应用。",
        }
    }

    pub fn opened_count(self, count: usize) -> String {
        match self {
            Self::En => format!("Opened {count} app(s)."),
            Self::Zh => format!("已打开 {count} 个应用。"),
        }
    }

    pub fn refuse_remove_original(self) -> &'static str {
        match self {
            Self::En => "Refusing to remove the original WeChat.app.",
            Self::Zh => "拒绝删除原版 WeChat.app。",
        }
    }

    pub fn removal_cancelled(self) -> &'static str {
        match self {
            Self::En => "Removal cancelled.",
            Self::Zh => "已取消删除。",
        }
    }

    pub fn removal_requires_confirmation(self) -> &'static str {
        match self {
            Self::En => "Removal requires confirmation.",
            Self::Zh => "删除操作需要确认。",
        }
    }

    pub fn removed(self) -> &'static str {
        match self {
            Self::En => "Removed",
            Self::Zh => "已删除",
        }
    }

    pub fn removal_danger(self) -> &'static str {
        match self {
            Self::En => "Danger: this will permanently delete the local WeChat copy.",
            Self::Zh => "危险：这将永久删除本地微信副本。",
        }
    }

    pub fn removal_prompt(self, path: &str) -> String {
        match self {
            Self::En => format!("Remove {path}? Type 'yes' to continue"),
            Self::Zh => format!("确认删除 {path} 吗？输入 yes 继续"),
        }
    }

    pub fn doctor_title(self) -> &'static str {
        match self {
            Self::En => "wetwin doctor",
            Self::Zh => "wetwin 环境检查",
        }
    }

    pub fn doctor_intro(self) -> &'static str {
        match self {
            Self::En => "Environment checks for macOS WeChat multi-instance management.",
            Self::Zh => "以下是 macOS 微信多开管理所需环境检查。",
        }
    }

    pub fn ok(self) -> &'static str {
        match self {
            Self::En => "OK",
            Self::Zh => "正常",
        }
    }

    pub fn warn(self) -> &'static str {
        match self {
            Self::En => "WARN",
            Self::Zh => "警告",
        }
    }

    pub fn doctor_running_on_macos(self) -> &'static str {
        match self {
            Self::En => "Running on macOS",
            Self::Zh => "当前系统为 macOS",
        }
    }

    pub fn doctor_original_exists(self) -> &'static str {
        match self {
            Self::En => "Original WeChat app exists",
            Self::Zh => "原版 WeChat.app 存在",
        }
    }

    pub fn doctor_plistbuddy_exists(self) -> &'static str {
        match self {
            Self::En => "PlistBuddy exists",
            Self::Zh => "PlistBuddy 可用",
        }
    }

    pub fn doctor_expected_source(self) -> &'static str {
        match self {
            Self::En => "Expected source app: /Applications/WeChat.app",
            Self::Zh => "期望的源应用路径：/Applications/WeChat.app",
        }
    }

    pub fn doctor_plistbuddy_note(self) -> &'static str {
        match self {
            Self::En => "/usr/libexec/PlistBuddy is required to change CFBundleIdentifier.",
            Self::Zh => "/usr/libexec/PlistBuddy 用于修改 CFBundleIdentifier。",
        }
    }

    pub fn doctor_command_available(self, command: &str) -> String {
        match self {
            Self::En => format!("Command available: {command}"),
            Self::Zh => format!("命令可用：{command}"),
        }
    }

    pub fn doctor_xattr_note(self) -> &'static str {
        match self {
            Self::En => "Required to clear quarantine attributes.",
            Self::Zh => "用于清除 quarantine 属性。",
        }
    }

    pub fn doctor_codesign_note(self) -> &'static str {
        match self {
            Self::En => "Required for local ad-hoc signing.",
            Self::Zh => "用于本地 ad-hoc 签名。",
        }
    }

    pub fn doctor_open_note(self) -> &'static str {
        match self {
            Self::En => "Required to launch app bundles.",
            Self::Zh => "用于启动应用。",
        }
    }

    pub fn doctor_osascript_note(self) -> &'static str {
        match self {
            Self::En => {
                "Used to request administrator privileges when /Applications needs elevated access."
            }
            Self::Zh => "当 /Applications 需要更高权限时，用于弹出管理员授权窗口。",
        }
    }

    pub fn doctor_can_write_applications(self) -> &'static str {
        match self {
            Self::En => "Can write to /Applications",
            Self::Zh => "可写入 /Applications",
        }
    }

    pub fn doctor_admin_prompt_note(self) -> &'static str {
        match self {
            Self::En => {
                "Some create/remove/sign operations may trigger a native macOS admin prompt."
            }
            Self::Zh => "某些创建、删除、签名操作可能会触发 macOS 原生管理员授权窗口。",
        }
    }

    pub fn doctor_no_local_copies(self) -> &'static str {
        match self {
            Self::En => "No local copies found.",
            Self::Zh => "未发现本地副本。",
        }
    }

    pub fn doctor_existing_copies(self) -> &'static str {
        match self {
            Self::En => "Existing copies",
            Self::Zh => "现有副本",
        }
    }

    pub fn doctor_copy_label(self, index: u16) -> String {
        match self {
            Self::En => format!("WeChat{index}"),
            Self::Zh => format!("微信副本 {index}"),
        }
    }

    pub fn error_label(self) -> &'static str {
        match self {
            Self::En => "Error",
            Self::Zh => "错误",
        }
    }

    pub fn invalid_yes_no(self) -> &'static str {
        match self {
            Self::En => "Please enter y/yes or n/no.",
            Self::Zh => "请输入 y/yes 或 n/no。",
        }
    }

    pub fn tui_title(self) -> &'static str {
        match self {
            Self::En => "wetwin TUI",
            Self::Zh => "wetwin 终端界面",
        }
    }

    pub fn tui_welcome(self) -> &'static str {
        match self {
            Self::En => "Welcome to wetwin. Use the menu on the left to manage WeChat copies.",
            Self::Zh => "欢迎使用 wetwin。请使用左侧菜单管理微信副本。",
        }
    }

    pub fn tui_actions(self) -> &'static str {
        match self {
            Self::En => "Actions",
            Self::Zh => "操作",
        }
    }

    pub fn tui_output(self) -> &'static str {
        match self {
            Self::En => "Output",
            Self::Zh => "输出",
        }
    }

    pub fn tui_output_welcome_title(self) -> &'static str {
        match self {
            Self::En => "Welcome",
            Self::Zh => "欢迎",
        }
    }

    pub fn tui_output_list_title(self) -> &'static str {
        match self {
            Self::En => "WeChat Apps",
            Self::Zh => "微信应用",
        }
    }

    pub fn tui_output_create_title(self) -> &'static str {
        match self {
            Self::En => "Create Copy",
            Self::Zh => "创建副本",
        }
    }

    pub fn tui_output_open_title(self) -> &'static str {
        match self {
            Self::En => "Open App",
            Self::Zh => "打开应用",
        }
    }

    pub fn tui_output_remove_title(self) -> &'static str {
        match self {
            Self::En => "Remove Copy",
            Self::Zh => "删除副本",
        }
    }

    pub fn tui_output_doctor_title(self) -> &'static str {
        match self {
            Self::En => "Environment Check",
            Self::Zh => "环境检查",
        }
    }

    pub fn tui_output_notice_title(self) -> &'static str {
        match self {
            Self::En => "Notice",
            Self::Zh => "提示",
        }
    }

    pub fn tui_output_error_title(self) -> &'static str {
        match self {
            Self::En => "Error",
            Self::Zh => "错误",
        }
    }

    pub fn tui_help(self) -> &'static str {
        match self {
            Self::En => "Up/Down: move  Enter: select  Esc/q: quit or close dialog",
            Self::Zh => "上下键：移动  回车：选择  Esc/q：退出或关闭弹窗",
        }
    }

    pub fn tui_requires_terminal(self) -> &'static str {
        match self {
            Self::En => "wetwin TUI must be run in a real terminal.",
            Self::Zh => "wetwin 的 TUI 需要在真实终端中运行。",
        }
    }

    pub fn tui_input(self) -> &'static str {
        match self {
            Self::En => "Input",
            Self::Zh => "输入",
        }
    }

    pub fn tui_modal_help(self) -> &'static str {
        match self {
            Self::En => "Enter to confirm, Esc to cancel",
            Self::Zh => "回车确认，Esc 取消",
        }
    }

    pub fn tui_create_title(self) -> &'static str {
        match self {
            Self::En => "Create Copy",
            Self::Zh => "创建副本",
        }
    }

    pub fn tui_open_title(self) -> &'static str {
        match self {
            Self::En => "Open Copy",
            Self::Zh => "打开副本",
        }
    }

    pub fn tui_remove_title(self) -> &'static str {
        match self {
            Self::En => "Remove Copy",
            Self::Zh => "删除副本",
        }
    }

    pub fn tui_remove_select_help(self) -> &'static str {
        match self {
            Self::En => "Use Up/Down to choose a copy, Enter to continue, Esc to cancel",
            Self::Zh => "使用上下键选择副本，回车继续，Esc 取消",
        }
    }

    pub fn tui_no_copies_to_remove(self) -> &'static str {
        match self {
            Self::En => "No WeChat copies are available to remove.",
            Self::Zh => "当前没有可删除的微信副本。",
        }
    }

    pub fn tui_no_copies_to_open(self) -> &'static str {
        match self {
            Self::En => "No WeChat copies are available to open.",
            Self::Zh => "当前没有可打开的微信副本。",
        }
    }

    pub fn tui_create_select_help(self) -> &'static str {
        match self {
            Self::En => {
                "Use Up/Down to choose an available copy slot, Enter to create, Esc to cancel"
            }
            Self::Zh => "使用上下键选择可用副本编号，回车创建，Esc 取消",
        }
    }

    pub fn tui_create_prompt(self, path: &str) -> String {
        match self {
            Self::En => format!("Create this WeChat copy now? [{path}]"),
            Self::Zh => format!("确认立即创建这个微信副本吗？ [{path}]"),
        }
    }

    pub fn tui_open_select_help(self) -> &'static str {
        match self {
            Self::En => "Use Up/Down to choose a copy, Enter to open, Esc to cancel",
            Self::Zh => "使用上下键选择副本，回车打开，Esc 取消",
        }
    }

    pub fn tui_confirm_title(self) -> &'static str {
        match self {
            Self::En => "Confirm",
            Self::Zh => "确认",
        }
    }

    pub fn tui_confirm_yes(self) -> &'static str {
        match self {
            Self::En => "Yes",
            Self::Zh => "是",
        }
    }

    pub fn tui_confirm_no(self) -> &'static str {
        match self {
            Self::En => "No",
            Self::Zh => "否",
        }
    }

    pub fn tui_confirm_help(self) -> &'static str {
        match self {
            Self::En => "Left/Right or Up/Down to choose, Enter to confirm, Esc to cancel",
            Self::Zh => "使用左右键或上下键选择，回车确认，Esc 取消",
        }
    }

    pub fn creation_cancelled(self) -> &'static str {
        match self {
            Self::En => "Copy creation was cancelled.",
            Self::Zh => "已取消创建副本。",
        }
    }

    pub fn tui_progress_title(self) -> &'static str {
        match self {
            Self::En => "Working",
            Self::Zh => "正在处理",
        }
    }

    pub fn tui_progress_help(self) -> &'static str {
        match self {
            Self::En => "Please wait while wetwin completes the operation.",
            Self::Zh => "wetwin 正在执行操作，请稍候。",
        }
    }

    pub fn tui_progress_starting(self, index: u16) -> String {
        match self {
            Self::En => format!("Preparing WeChat{index}.app"),
            Self::Zh => format!("正在准备 WeChat{index}.app"),
        }
    }

    pub fn tui_background_disconnected(self) -> &'static str {
        match self {
            Self::En => "The background task ended unexpectedly.",
            Self::Zh => "后台任务意外中断。",
        }
    }

    pub fn tui_next_index_save_failed(self, details: &str) -> String {
        match self {
            Self::En => format!("Copy created, but updating the next copy index failed: {details}"),
            Self::Zh => format!("副本已创建，但更新下一个副本编号失败：{details}"),
        }
    }

    pub fn tui_menu_items(self) -> [&'static str; 8] {
        match self {
            Self::En => [
                "List WeChat apps",
                "Create a copy",
                "Open one copy",
                "Open all apps",
                "Remove a copy",
                "Run doctor",
                "Change language",
                "Exit",
            ],
            Self::Zh => [
                "查看微信应用",
                "创建副本",
                "打开单个副本",
                "打开全部应用",
                "删除副本",
                "运行环境检查",
                "切换语言",
                "退出",
            ],
        }
    }

    pub fn tui_language_title(self) -> &'static str {
        match self {
            Self::En => "Language",
            Self::Zh => "语言",
        }
    }

    pub fn tui_language_help(self) -> &'static str {
        match self {
            Self::En => "Use Up/Down to choose language, Enter to confirm, Esc to cancel",
            Self::Zh => "使用上下键选择语言，回车确认，Esc 取消",
        }
    }

    pub fn tui_language_saved(self) -> &'static str {
        match self {
            Self::En => "Language updated and saved.",
            Self::Zh => "语言已更新并保存。",
        }
    }

    pub fn tui_language_save_failed(self, details: &str) -> String {
        match self {
            Self::En => format!("Language changed, but saving config failed: {details}"),
            Self::Zh => format!("语言已切换，但保存配置失败：{details}"),
        }
    }

    pub fn tui_language_choices(self) -> [&'static str; 2] {
        match self {
            Self::En | Self::Zh => ["中文", "English"],
        }
    }
}
