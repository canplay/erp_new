# -*- coding: utf-8 -*-
"""批量生成 MyAI 功能演示静态页（供高清截图）"""
import os

STYLE = """<style>
  body { font-family: "微软雅黑", sans-serif; background: #f5f7fa; margin: 0; padding: 20px; }
  .header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
  .header h2 { margin: 0; color: #1f2d3d; }
  .header .sub { color: #909399; font-size: 13px; }
  .cards { display: flex; gap: 16px; margin-bottom: 16px; }
  .card { background: #fff; border-radius: 8px; padding: 14px 22px; box-shadow: 0 1px 4px rgba(0,0,0,.08); flex: 1; }
  .card .n { font-size: 26px; font-weight: bold; } .card .l { color: #909399; font-size: 13px; margin-top: 4px; }
  table { width: 100%; border-collapse: collapse; background: #fff; border-radius: 8px; overflow: hidden; box-shadow: 0 1px 4px rgba(0,0,0,.08); }
  th, td { padding: 11px 14px; text-align: left; border-bottom: 1px solid #ebeef5; font-size: 13px; }
  th { background: #f8f9fb; color: #606266; font-weight: 600; }
  .badge { display: inline-block; padding: 2px 9px; border-radius: 10px; font-size: 12px; color: #fff; }
  .b1 { background: #22b14c; } .b2 { background: #409eff; } .b3 { background: #909399; } .b4 { background: #e64545; } .b5 { background: #f58220; }
  .foot { margin-top: 12px; color: #909399; font-size: 13px; }
</style>"""

def page(title, kpis, table_head, rows, foot, icon="📄"):
    trs = "".join(f"<tr>{''.join(f'<td>{c}</td>' for c in r)}</tr>" for r in rows)
    kpi_html = "".join(f'<div class="card"><div class="n" style="color:{c}">{n}</div><div class="l">{l}</div></div>'
                       for n, l, c in kpis)
    heads = "".join(f"<th>{h}</th>" for h in table_head)
    return f"""<!DOCTYPE html>
<html lang="zh-CN"><head><meta charset="utf-8"><title>{title}</title>{STYLE}</head>
<body>
  <div class="header"><h2>{icon} MyAI · {title}</h2><span class="sub">示例城市 · 演示数据</span></div>
  <div class="cards">{kpi_html}</div>
  <table><thead><tr>{heads}</tr></thead><tbody>{trs}</tbody></table>
  <div class="foot">{foot}</div>
</body></html>"""

pages = {
"file.html": page("文件管理", [("12,846", "文件总数", "#409eff"), ("86.4 GB", "存储占用", "#22b14c"), ("328", "今日上传", "#f58220")],
 ["文件名称", "类型", "大小", "上传者", "时间", "状态"],
 [["电单车监管月度报表.pdf", "PDF", "2.4 MB", "admin", "2026-08-28 15:42", '<span class="badge b1">正常</span>'],
  ["车辆GPS轨迹回放_20260827.mp4", "视频", "86.5 MB", "operator", "2026-08-27 20:15", '<span class="badge b1">正常</span>'],
  ["城市投放规划示意图.png", "图片", "1.8 MB", "analyst", "2026-08-27 11:30", '<span class="badge b1">正常</span>'],
  ["运营商合作协议-青桔.docx", "文档", "356 KB", "auditor", "2026-08-26 17:05", '<span class="badge b1">正常</span>'],
  ["超速告警录音_000231.wav", "音频", "1.2 MB", "support", "2026-08-26 09:48", '<span class="badge b5">待审</span>'],
  ["巡检日报_20260825.xlsx", "表格", "128 KB", "guard", "2026-08-25 18:20", '<span class="badge b1">正常</span>'],
  ["站点照片_示例.zip", "压缩包", "24.6 MB", "dispatcher", "2026-08-25 10:12", '<span class="badge b1">正常</span>']],
 "文件类型过滤 · 上传/预览/分享 · 云存储服务", "🗂️"),

"cms.html": page("内容管理", [("326", "内容总数", "#409eff"), ("18", "今日发布", "#22b14c"), ("42.8k", "本月阅读", "#f58220")],
 ["标题", "栏目", "状态", "阅读量", "发布时间"],
 [["共享电单车规范停放倡议书", "公告", '<span class="badge b1">已发布</span>', "12,860", "2026-08-28 09:00"],
  ["示例城市电单车管理办法解读", "政策", '<span class="badge b1">已发布</span>', "8,432", "2026-08-26 14:30"],
  ["安全骑行温馨提示（雨季版）", "安全", '<span class="badge b1">已发布</span>', "6,120", "2026-08-24 10:15"],
  ["充电桩选址公示（二期）", "公告", '<span class="badge b5">待审核</span>', "0", "2026-08-28 11:20"],
  ["运营商月度考核结果", "考核", '<span class="badge b2">草稿</span>', "0", "2026-08-27 16:45"],
  ["头盔佩戴抽查专项行动", "安全", '<span class="badge b1">已发布</span>', "4,876", "2026-08-22 09:30"]],
 "CMS 内容发布 · 多栏目 · 审核流", "📰"),

"orders.html": page("订单管理", [("128,650", "本月订单", "#409eff"), ("¥384,200", "本月流水", "#22b14c"), ("98.2%", "完成率", "#f58220")],
 ["订单号", "车辆", "用户", "金额", "状态", "时间"],
 [["ORD-20260828-00921", "MYK-0001", "138****2210", "¥3.50", '<span class="badge b1">已完成</span>', "2026-08-28 16:32"],
  ["ORD-20260828-00922", "MYK-0009", "187****0456", "¥2.00", '<span class="badge b1">已完成</span>', "2026-08-28 16:30"],
  ["ORD-20260828-00923", "MYK-0005", "139****8831", "¥1.50", '<span class="badge b2">骑行中</span>', "2026-08-28 16:28"],
  ["ORD-20260828-00924", "MYK-0007", "158****6677", "¥4.00", '<span class="badge b1">已完成</span>', "2026-08-28 16:25"],
  ["ORD-20260828-00925", "MYK-0012", "186****3345", "¥2.50", '<span class="badge b4">已退款</span>', "2026-08-28 16:22"],
  ["ORD-20260828-00926", "MYK-0004", "135****9902", "¥3.00", '<span class="badge b1">已完成</span>', "2026-08-28 16:18"]],
 "订单全流程 · 支付/退款/结算 · 实时流水", "🧾"),

"audit.html": page("审计日志", [("56,230", "审计记录", "#409eff"), ("1,208", "今日操作", "#22b14c"), ("0", "安全事件", "#22b14c")],
 ["操作人", "操作", "模块", "IP", "时间"],
 [["admin", "登录系统", "身份认证", "127.0.0.1", "2026-08-28 16:31:02"],
  ["operator", "更新车辆 MYK-0005 状态", "电单车监管", "192.0.2.21", "2026-08-28 16:27:44"],
  ["auditor", "导出月度审计报表", "审计", "192.0.2.14", "2026-08-28 16:15:20"],
  ["dispatcher", "调度车辆 8 台至中心区", "电单车监管", "192.0.2.30", "2026-08-28 15:52:11"],
  ["admin", "配置电子围栏（示例位置 500m）", "电单车监管", "127.0.0.1", "2026-08-28 15:31:26"],
  ["support", "处理用户投诉 #4821", "客服", "192.0.2.9", "2026-08-28 15:10:45"],
  ["system", "超速告警触发（MYK-0010）", "告警服务", "192.0.2.5", "2026-08-28 15:08:33"]],
 "全量操作审计 · 合规追溯 · 安全事件零容忍", "🔍"),

"messages.html": page("消息中心", [("38", "未读消息", "#e64545"), ("1,240", "本月消息", "#409eff"), ("6", "告警消息", "#f58220")],
 ["类型", "标题", "接收人", "时间", "状态"],
 [['<span class="badge b4">告警</span>', "MYK-0010 超速告警（20.1 km/h）", "operator", "2026-08-28 15:08", '<span class="badge b4">未读</span>'],
  ['<span class="badge b4">告警</span>', "MYK-0008 设备失联超 1 小时", "dispatcher", "2026-08-28 14:55", '<span class="badge b4">未读</span>'],
  ['<span class="badge b2">系统</span>', "电子围栏配置已更新", "admin", "2026-08-28 15:31", '<span class="badge b1">已读</span>'],
  ['<span class="badge b5">运营</span>', "本月运营商考核数据已生成", "auditor", "2026-08-28 09:00", '<span class="badge b1">已读</span>'],
  ['<span class="badge b2">系统</span>', "文件备份任务完成（86.4GB）", "admin", "2026-08-28 02:00", '<span class="badge b1">已读</span>'],
  ['<span class="badge b3">通知</span>', "安全骑行倡议书已发布", "全部用户", "2026-08-28 09:00", '<span class="badge b1">已读</span>']],
 "站内信 · 告警推送 · 已读回执", "🔔"),

"workflow.html": page("工作流", [("86", "运行中流程", "#409eff"), ("1,320", "本月办结", "#22b14c"), ("4.2h", "平均耗时", "#f58220")],
 ["流程名称", "发起人", "当前节点", "状态", "发起时间"],
 [["电单车投放审批", "operator", "运营主管审批", '<span class="badge b2">审批中</span>', "2026-08-28 14:20"],
  ["围栏调整申请（示例位置）", "dispatcher", "管理员审批", '<span class="badge b2">审批中</span>', "2026-08-28 13:45"],
  ["运营商月度考核复核", "auditor", "归档", '<span class="badge b1">已完成</span>', "2026-08-28 10:00"],
  ["采购申请（定位终端 500 台）", "operator", "财务审批", '<span class="badge b2">审批中</span>', "2026-08-27 16:30"],
  ["新站点开通（示例区域）", "dispatcher", "归档", '<span class="badge b1">已完成</span>', "2026-08-27 11:10"],
  ["超速告警处置单 MYK-0010", "system", "处置完成", '<span class="badge b1">已办结</span>', "2026-08-28 15:09"]],
 "可视化流程 · 审批链 · 自动流转", "🔄"),

"hik.html": page("视频监控", [("128", "接入摄像头", "#409eff"), ("126", "在线", "#22b14c"), ("98.4%", "在线率", "#22b14c")],
 ["点位名称", "区域", "状态", "清晰度", "最后预览"],
 [["示例站点A", "中心城区", '<span class="badge b1">在线</span>', "1080P", "2026-08-28 16:31:58"],
  ["示例站点B", "中心城区", '<span class="badge b1">在线</span>', "1080P", "2026-08-28 16:31:57"],
  ["示例站点C", "学府片区", '<span class="badge b1">在线</span>', "720P", "2026-08-28 16:31:55"],
  ["示例站点D", "河畔片区", '<span class="badge b4">离线</span>', "720P", "2026-08-28 14:02:10"],
  ["开化中路", "中心城区", '<span class="badge b1">在线</span>', "1080P", "2026-08-28 16:31:52"],
  ["三七产业园区", "园区", '<span class="badge b1">在线</span>', "720P", "2026-08-28 16:31:48"]],
 "海康视频接入 · 实时预览 · 点位管理", "🎥"),

"lpr.html": page("车牌识别", [("18,640", "今日识别", "#409eff"), ("96.8%", "识别率", "#22b14c"), ("12", "违停识别", "#f58220")],
 ["车牌号", "识别时间", "地点", "车辆类型", "状态"],
 [["云A·00001", "2026-08-28 16:31:22", "示例站点A", "电动自行车", '<span class="badge b1">正常</span>'],
  ["云H·67890", "2026-08-28 16:30:58", "开化中路", "电动自行车", '<span class="badge b5">违停</span>'],
  ["云A·00002", "2026-08-28 16:30:41", "示例站点C", "电动自行车", '<span class="badge b1">正常</span>'],
  ["云A·00003", "2026-08-28 16:29:55", "示例站点D", "电动自行车", '<span class="badge b1">正常</span>'],
  ["云H·97531", "2026-08-28 16:29:12", "三七产业园区", "电动自行车", '<span class="badge b5">违停</span>'],
  ["云A·00004", "2026-08-28 16:28:47", "示例站点B", "电动自行车", '<span class="badge b1">正常</span>']],
 "LPR 车牌识别 · 违停检测 · 通行统计", "🚗"),
}

out_dir = "D:/Workspace/MyAI/Frontend/admin/dist/spa/demo"
for name, html in pages.items():
    with open(os.path.join(out_dir, name), "w", encoding="utf-8") as f:
        f.write(html)
    print("written", name)
print("共", len(pages), "个新页面")
