#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""生成 MyAI 智能管理平台 + ERP 多业务平台 项目汇报 docx。"""

import os

from docx import Document
from docx.enum.text import WD_ALIGN_PARAGRAPH
from docx.oxml.ns import qn
from docx.shared import Cm, Pt, RGBColor

BASE_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DOCS_DIR = os.path.join(BASE_DIR, "docs")
SHOT_DIR = os.path.join(DOCS_DIR, "汇报截图")
OUT_PATH = os.path.join(DOCS_DIR, "项目汇报.docx")

IMG_WIDTH = Cm(16)
ZH_FONT = "微软雅黑"
EN_FONT = "Calibri"


def set_zh_font(run, size=None, bold=None, color=None):
    run.font.name = EN_FONT
    r = run._element.rPr.rFonts
    r.set(qn("w:eastAsia"), ZH_FONT)
    if size is not None:
        run.font.size = Pt(size)
    if bold is not None:
        run.font.bold = bold
    if color is not None:
        run.font.color.rgb = RGBColor(*color)


def add_para(doc, text, size=11, bold=False, align=None, space_after=6, color=None, indent=None):
    p = doc.add_paragraph()
    if align is not None:
        p.alignment = align
    p.paragraph_format.space_after = Pt(space_after)
    p.paragraph_format.space_before = Pt(0)
    if indent is not None:
        p.paragraph_format.first_line_indent = Pt(indent)
    run = p.add_run(text)
    set_zh_font(run, size=size, bold=bold, color=color)
    return p


def add_table(doc, headers, rows, widths=None):
    table = doc.add_table(rows=1, cols=len(headers))
    table.style = "Table Grid"
    hdr = table.rows[0].cells
    for i, h in enumerate(headers):
        hdr[i].text = ""
        run = hdr[i].paragraphs[0].add_run(h)
        set_zh_font(run, size=10.5, bold=True)
    for row in rows:
        cells = table.add_row().cells
        for i, val in enumerate(row):
            cells[i].text = ""
            run = cells[i].paragraphs[0].add_run(val)
            set_zh_font(run, size=10.5)
    if widths:
        for i, w in enumerate(widths):
            for row in table.rows:
                row.cells[i].width = Cm(w)
    return table


def add_picture_centered(doc, filename, caption=None):
    path = os.path.join(SHOT_DIR, filename)
    if not os.path.exists(path):
        add_para(doc, f"[缺失图片：{filename}，请补充后再生成]", size=10.5, color=(255, 0, 0))
        return False
    doc.add_picture(path, width=IMG_WIDTH)
    doc.paragraphs[-1].alignment = WD_ALIGN_PARAGRAPH.CENTER
    if caption:
        add_para(doc, caption, size=9.5, align=WD_ALIGN_PARAGRAPH.CENTER,
                 space_after=10, color=(90, 90, 90))
    return True


def build():
    doc = Document()

    # 封面
    for _ in range(6):
        doc.add_paragraph()
    add_para(doc, "MyAI 智能管理平台 + ERP 多业务平台", size=26, bold=True,
             align=WD_ALIGN_PARAGRAPH.CENTER, space_after=8)
    add_para(doc, "项 目 汇 报", size=20, bold=True,
             align=WD_ALIGN_PARAGRAPH.CENTER, space_after=16)
    add_para(doc, "—— 电单车监管解决方案 ——", size=14,
             align=WD_ALIGN_PARAGRAPH.CENTER, space_after=0)
    doc.add_page_break()

    # 一、项目概述
    doc.add_heading("一、项目概述", level=1)
    add_para(doc,
             "MyAI 智能管理平台基于 Rust 构建 21 个 gRPC 微服务，采用 Vue3/Quasar 打造"
             "管理端、运维端、社交端三端应用，通过 K8s/Rancher 部署，提供高性能、可伸缩的企业级底座。")
    add_para(doc,
             "ERP 多业务平台面向多语言、多业务线场景，覆盖电单车监管、拖车服务、殡葬服务等"
             "多条业务线，支持按行业定制与扩展。")
    add_para(doc,
             "一句话定位：以微服务底座 + 多业务平台，为城市出行与公共服务提供一站式数字化监管与运营能力。",
             bold=True)

    # 二、功能介绍与业务场景
    doc.add_heading("二、功能介绍与业务场景", level=1)

    doc.add_heading("2.1 MyAI 服务分类与业务场景", level=2)
    add_table(
        doc,
        ["分类", "服务", "业务场景一句话"],
        [
            ["核心服务", "身份 / 用户 / 租户 / 文件 / CMS",
             "统一账号体系与租户隔离，多平台共用一套用户与内容管理"],
            ["业务服务", "电单车 / 拖车 / 支付 / 工作流",
             "覆盖核心业务办理与运营流程，业务可插拔扩展"],
            ["智能服务", "海康视频 / 车牌识别 / 短信",
             "视频接入、车牌识别、消息触达，提升监管自动化水平"],
            ["运营服务", "审计 / 反馈 / 社交运营",
             "审计留痕、用户反馈闭环与社区化运营"],
        ],
        widths=[2.4, 6.6, 7.0],
    )
    doc.add_paragraph()

    doc.add_heading("2.2 ERP 多业务线列表", level=2)
    add_table(
        doc,
        ["业务线", "定位"],
        [
            ["电单车监管", "车辆档案、实时定位、超速报警、电子围栏、电池库存、运营报表（本期重点）"],
            ["拖车服务", "拖车调度与订单全流程管理"],
            ["殡葬服务", "殡葬业务登记与流程管理"],
        ],
        widths=[3.0, 13.0],
    )
    doc.add_page_break()

    # 平台截图（其他前端功能演示——真实页面）
    doc.add_heading("2.3 平台功能展示", level=2)
    for img, cap in [
        ("myai-dashboard.jpg", "图：平台总览（多租户 · 微服务状态）"),
        ("myai-users.jpg", "图：用户管理（多租户 RBAC）"),
        ("myai-tenant.jpg", "图：租户管理（多租户隔离）"),
        ("myai-files.jpg", "图：文件管理（云存储）"),
        ("myai-cms.jpg", "图：内容管理（CMS 栏目 · 文章）"),
        ("myai-audit.jpg", "图：操作审计（合规追溯）"),
        ("myai-messages.jpg", "图：消息中心（告警推送）"),
        ("myai-workflow.jpg", "图：工作流（流程审批）"),
        ("myai-lpr.jpg", "图：车牌识别（LPR 通行记录）"),
    ]:
        add_picture_centered(doc, img, cap)
        doc.paragraphs[-1].paragraph_format.space_after = Pt(6)
    doc.add_page_break()

    # 三、电单车监管（重点）
    doc.add_heading("三、电单车监管", level=1)

    doc.add_heading("3.1 监管痛点", level=2)
    add_para(doc,
             "传统电单车管理依赖人工台账，存在车辆底数不清、失窃追查难、超速无预警、"
             "合规监管缺数据支撑等痛点。信息分散、响应滞后，监管成本高而效率低。")

    doc.add_heading("3.2 核心功能", level=2)
    features = [
        ("车辆档案", "统一建立车辆电子档案，车牌、车架、电池、运营商等信息一车一档，底数清晰可查。",
         "ebike-monitor.jpg", "图：电单车车辆管理（实时列表 · 搜索 · 地图）"),
        ("实时定位地图", "接入 GPS 定位与地图组件，实时展示车辆分布与运行状态，一目了然。",
         "ebike-car-list.jpg", "图：电单车车辆列表（状态/速度/警告）"),
        ("速度监测报警", "超速实时监测并触发告警，车辆详情联动告警信息，异常分钟级响应。",
         "ebike-reports.jpg", "图：电单车数据报表（运营统计）"),
        ("电池库存", "电池资产入库、领用、更换全程记录，库存与状态实时掌握。",
         None, None),
        ("订单运营", "电单车业务订单全流程管理，支撑日常运营与结算。",
         None, None),
        ("多运营商接入", "哈啰/青桔/美团等多运营商车辆统一接入监管，覆盖全量在网车辆。",
         None, None),
    ]
    for title, desc, img, cap in features:
        add_para(doc, f"（{title}）{desc}", bold=False, space_after=4)
        if img:
            add_picture_centered(doc, img, cap)
            doc.paragraphs[-1].paragraph_format.space_after = Pt(6)

    doc.add_heading("3.3 技术亮点", level=2)
    for item in [
        "gRPC 微服务架构：21 个微服务独立部署、弹性伸缩，支撑高并发监管业务。",
        "高德地图 GPS：定位与轨迹能力，监管可视化呈现。",
        "电子围栏：划定限行区域，越界实时告警。",
        "多运营商接入：兼容多厂商设备与数据接入。",
        "一车一码：车辆唯一标识，扫码即查，全生命周期可追溯。",
    ]:
        add_para(doc, f"• {item}", space_after=3)
    doc.add_page_break()

    # 四、预期未来收益
    doc.add_heading("四、预期未来收益", level=1)

    doc.add_heading("4.1 定性收益", level=2)
    for item in [
        "监管数字化：台账电子化、数据自动归集，管理底数清晰。",
        "安全提升：超速预警与电子围栏减少事故与违规风险。",
        "合规可追溯：车辆全生命周期留痕，满足监管审计要求。",
        "决策支持：多维报表为运营与监管决策提供数据依据。",
    ]:
        add_para(doc, f"• {item}", space_after=3)

    doc.add_heading("4.2 量化预期（保守区间）", level=2)
    add_table(
        doc,
        ["指标", "预期"],
        [
            ["监管人力成本", "下降 30% ~ 50%"],
            ["异常响应时间", "由小时级降至分钟级"],
            ["车辆失窃率", "显著下降（实时定位 + 电子围栏）"],
            ["运营效率", "提升（一车一码 + 自动报表）"],
        ],
        widths=[4.0, 12.0],
    )
    add_para(doc, "注：以上为预期区间，具体收益需结合实际部署规模与运营情况进行评估。",
             size=9.5, color=(90, 90, 90))
    doc.add_page_break()

    # 五、总结与展望
    doc.add_heading("五、总结与展望", level=1)
    add_para(doc,
             "一期已基于 K8s 完成平台与电单车监管核心能力上线，实现车辆档案、实时定位、"
             "超速报警、数据报表等能力的落地。")
    add_para(doc,
             "二期将重点扩展 AI 视频分析（违规行为识别）、预测性维护（电池/车辆健康评估）"
             "与跨区域复制能力，持续深化电单车监管与多业务平台价值。")

    # 附：插图清单
    doc.add_heading("附：插图清单（12 张）", level=1)
    imgs = [
        "ebike-monitor.jpg — 电单车车辆管理（实时列表 · 地图）",
        "ebike-car-list.jpg — 电单车车辆列表（状态/速度/警告）",
        "ebike-reports.jpg — 电单车数据报表",
        "myai-dashboard.jpg — 平台总览",
        "myai-users.jpg — 用户管理（RBAC）",
        "myai-tenant.jpg — 租户管理",
        "myai-files.jpg — 文件管理",
        "myai-cms.jpg — 内容管理（CMS）",
        "myai-audit.jpg — 操作审计",
        "myai-messages.jpg — 消息中心",
        "myai-workflow.jpg — 工作流",
        "myai-lpr.jpg — 车牌识别（LPR）",
    ]
    for it in imgs:
        add_para(doc, f"• {it}", space_after=3)

    os.makedirs(DOCS_DIR, exist_ok=True)
    doc.save(OUT_PATH)
    return doc


if __name__ == "__main__":
    d = build()
    para_count = len(d.paragraphs)
    print(f"生成成功：{OUT_PATH}")
    print(f"段落数：{para_count}")
    missing = [f for f in ["ebike-monitor.jpg", "ebike-car-list.jpg", "ebike-reports.jpg",
                           "myai-dashboard.jpg", "myai-users.jpg", "myai-tenant.jpg",
                           "myai-files.jpg", "myai-cms.jpg", "myai-audit.jpg",
                           "myai-messages.jpg", "myai-workflow.jpg", "myai-lpr.jpg"]
               if not os.path.exists(os.path.join(SHOT_DIR, f))]
    if missing:
        print(f"警告：以下截图缺失，对应插图已标注占位：{missing}")
