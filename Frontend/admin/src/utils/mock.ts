/**
 * @file mock.ts
 * @description 开发演示模式 (mock 数据层)
 * 仅供本地截图 / 演示使用。构建时设置 VITE_USE_MOCK=true 开启;
 * 正式构建不携带该变量时 isMockMode() 恒为 false, 完全不影响真实请求链路。
 */

// ==================== 模式开关 ====================

export function isMockMode(): boolean {
  return import.meta.env.VITE_USE_MOCK === 'true';
}

// ==================== 类型 ====================

export interface MockMatchResult {
  status: number;
  body: Record<string, unknown>;
}

// ==================== 工具函数 ====================

/** 确定性伪随机数生成器, 保证演示数据可复现 */
function mulberry32(seed: number): () => number {
  let a = seed;
  return () => {
    a |= 0;
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function roundNum(value: number, digits: number): number {
  const factor = 10 ** digits;
  return Math.round(value * factor) / factor;
}

function pad2(n: number): string {
  return String(n).padStart(2, '0');
}

function fmtTime(d: Date): string {
  return (
    `${d.getFullYear()}-${pad2(d.getMonth() + 1)}-${pad2(d.getDate())} ` +
    `${pad2(d.getHours())}:${pad2(d.getMinutes())}:${pad2(d.getSeconds())}`
  );
}

function toEpochSeconds(d: Date): number {
  return Math.floor(d.getTime() / 1000);
}

/** 统一成功响应体, 由 alova transformResponse 解包 */
function ok(data: unknown): MockMatchResult {
  return { status: 200, body: { code: 200, message: 'success', success: true, data } };
}

/** 安全地取字符串字段 */
function asString(value: unknown, fallback = ''): string {
  return typeof value === 'string' ? value : fallback;
}

// ==================== 首页 KPI ====================

export const mockDashboardKpi = {
  statistics: {
    total_users: 8,
    active_users: 7,
    new_users_today: 2,
    login_attempts: 56,
    failed_logins: 3,
    successful_logins: 53,
  },
  user_growth: [
    { label: '08-22', value: 2 },
    { label: '08-23', value: 3 },
    { label: '08-24', value: 1 },
    { label: '08-25', value: 4 },
    { label: '08-26', value: 2 },
    { label: '08-27', value: 5 },
    { label: '08-28', value: 3 },
  ],
  login_types: [
    { label: '密码登录', value: 42 },
    { label: '短信验证码', value: 11 },
    { label: 'OAuth 单点登录', value: 3 },
  ],
};

// ==================== 用户列表 (8 个) ====================

export const mockUsers = [
  { id: 1, username: 'admin', nickname: '监管员', email: 'admin@example.com', phone: '13800000001', role: 'admin', status: 1, created_at: '2026-01-05 10:00:00', last_login_at: '2026-08-28 09:00:00' },
  { id: 2, username: 'operator', nickname: '用户乙', email: 'operator@example.com', phone: '13800000002', role: 'operator', status: 1, created_at: '2026-02-11 14:30:00', last_login_at: '2026-08-27 18:22:00' },
  { id: 3, username: 'auditor', nickname: '用户丙', email: 'auditor@example.com', phone: '13800000003', role: 'auditor', status: 1, created_at: '2026-03-03 09:15:00', last_login_at: '2026-08-28 08:10:00' },
  { id: 4, username: 'analyst', nickname: '用户丁', email: 'analyst@example.com', phone: '13800000004', role: 'analyst', status: 1, created_at: '2026-04-19 11:45:00', last_login_at: '2026-08-26 16:40:00' },
  { id: 5, username: 'dispatcher', nickname: '用户戊', email: 'dispatcher@example.com', phone: '13800000005', role: 'dispatcher', status: 1, created_at: '2026-05-22 08:20:00', last_login_at: '2026-08-28 07:55:00' },
  { id: 6, username: 'support', nickname: '用户己', email: 'support@example.com', phone: '13800000006', role: 'support', status: 1, created_at: '2026-06-15 13:10:00', last_login_at: '2026-08-25 20:05:00' },
  { id: 7, username: 'guard', nickname: '用户庚', email: 'guard@example.com', phone: '13800000007', role: 'guard', status: 1, created_at: '2026-07-08 10:50:00', last_login_at: '2026-08-24 22:30:00' },
  { id: 8, username: 'tester', nickname: '测试用户', email: 'tester@example.com', phone: '13800000008', role: 'user', status: 0, created_at: '2026-08-01 17:00:00', last_login_at: '2026-08-20 12:00:00' },
];

// ==================== 订单 (6 条) ====================

const nowMs = Date.now();

export const mockOrders = [
  { code: 'MYK-0001', type: '骑行订单', provide: '哈啰', status: 0, speed: 12.5, time: fmtTime(new Date(nowMs - 20 * 60000)), alert: '', remark: '示例路线' },
  { code: 'MYK-0004', type: '骑行订单', provide: '哈啰', status: 0, speed: 8.2, time: fmtTime(new Date(nowMs - 55 * 60000)), alert: '超速预警', remark: '示例区域' },
  { code: 'MYK-0007', type: '骑行订单', provide: '美团', status: 0, speed: 15.8, time: fmtTime(new Date(nowMs - 90 * 60000)), alert: '', remark: '示例路段' },
  { code: 'MYK-0011', type: '骑行订单', provide: '青桔', status: 0, speed: 20.1, time: fmtTime(new Date(nowMs - 130 * 60000)), alert: '', remark: '示例方向' },
  { code: 'MYK-0002', type: '关停订单', provide: '青桔', status: 1, speed: 0, time: fmtTime(new Date(nowMs - 3 * 3600000)), alert: '', remark: '订单正常关闭' },
  { code: 'MYK-0006', type: '故障订单', provide: '美团', status: 2, speed: 0, time: fmtTime(new Date(nowMs - 26 * 3600000)), alert: '车辆故障', remark: '已派工处理' },
];

// ==================== 存储/停放区 (5 个) ====================

function polygonAround(lng: number, lat: number): Array<Array<number>> {
  return [
    [lng - 0.0012, lat - 0.0009],
    [lng + 0.0012, lat - 0.0009],
    [lng + 0.0012, lat + 0.0009],
    [lng - 0.0012, lat + 0.0009],
  ];
}

export const mockStorages = [
  { code: 'WSC-01', provide: '哈啰', status: 1, cur: 23, sum: 100, gps: { lng: 104.2515, lat: 23.384 }, points: polygonAround(104.0000, 23.0000), total_capacity: 100, current_count: 23, capacity: '23/100', condition: '启用', location: '示例站点A', alert: '', remark: '演示数据' },
  { code: 'WSC-02', provide: '青桔', status: 1, cur: 46, sum: 120, gps: { lng: 104.2436, lat: 23.3876 }, points: polygonAround(104.1000, 23.1000), total_capacity: 120, current_count: 46, capacity: '46/120', condition: '启用', location: '示例站点B', alert: '', remark: '演示数据' },
  { code: 'WSC-03', provide: '美团', status: 1, cur: 12, sum: 80, gps: { lng: 104.2561, lat: 23.3809 }, points: polygonAround(104.2000, 23.2000), total_capacity: 80, current_count: 12, capacity: '12/80', condition: '启用', location: '示例站点C', alert: '', remark: '演示数据' },
  { code: 'WSC-04', provide: '哈啰', status: 2, cur: 5, sum: 60, gps: { lng: 104.2345, lat: 23.3768 }, points: polygonAround(104.3000, 23.3000), total_capacity: 60, current_count: 5, capacity: '5/60', condition: '维护', location: '示例站点D', alert: '', remark: '演示数据' },
  { code: 'WSC-05', provide: '青桔', status: 1, cur: 31, sum: 90, gps: { lng: 104.2621, lat: 23.3905 }, points: polygonAround(104.4000, 23.4000), total_capacity: 90, current_count: 31, capacity: '31/90', condition: '启用', location: '示例站点E', alert: '', remark: '演示数据' },
];

// ==================== 电单车 (12 台) ====================

export interface MockCar {
  code: string;
  provide: string;
  status: number;
  speed: number;
  gps: { lng: number; lat: number };
  time: { start: string; end: string };
  alert: string;
  remark: string;
  gps_type: number;
  create_date?: string;
}

function generateMockEbikes(): MockCar[] {
  const provides = ['哈啰', '青桔', '美团'];
  const statuses = [2, 1, 0, 2, 1, 3, 2, 2, 1, 0, 2, 1];
  const alertFlags = [false, false, false, true, false, true, false, true, false, false, true, false];
  const rand = mulberry32(20260828);
  const cars: MockCar[] = [];

  for (let i = 0; i < 12; i++) {
    const lng = roundNum(104.0 + (rand() - 0.5) * 0.05, 6);
    const lat = roundNum(23.0 + (rand() - 0.5) * 0.05, 6);
    const status = statuses[i] ?? 1;
    const running = status === 2;
    const speed = running ? roundNum(rand() * 25, 1) : 0;
    const alerted = alertFlags[i] ?? false;
    const alert = alerted ? (status === 3 ? '车辆故障' : '超速预警') : '';
    const startMs = nowMs - (15 + Math.floor(rand() * 120)) * 60000;
    const endMs = nowMs - Math.floor(rand() * 10) * 60000;

    cars.push({
      code: `MYK-${String(i + 1).padStart(4, '0')}`,
      provide: provides[i % provides.length] ?? '哈啰',
      status,
      speed,
      gps: { lng, lat },
      time: { start: fmtTime(new Date(startMs)), end: fmtTime(new Date(endMs)) },
      alert,
      remark: alerted ? '演示数据：车辆存在告警' : '',
      gps_type: 1,
    });
  }

  return cars;
}

export const mockEbikes: MockCar[] = generateMockEbikes();

// ==================== 登录日志 ====================

export const mockLoginLogs = [
  { id: 1, user_id: 1, username: 'admin', loginType: '密码登录', ip: '127.0.0.1', location: '本机', success: true, status: 1, created_at: toEpochSeconds(new Date(nowMs - 8 * 60000)) },
  { id: 2, user_id: 5, username: 'dispatcher', loginType: '密码登录', ip: '192.0.2.21', location: '示例地区', success: true, status: 1, created_at: toEpochSeconds(new Date(nowMs - 26 * 60000)) },
  { id: 3, user_id: 3, username: 'auditor', loginType: '密码登录', ip: '192.0.2.14', location: '示例地区', success: true, status: 1, created_at: toEpochSeconds(new Date(nowMs - 50 * 60000)) },
  { id: 4, user_id: 2, username: 'operator', loginType: '短信验证码', ip: '192.0.2.8', location: '示例地区', success: true, status: 1, created_at: toEpochSeconds(new Date(nowMs - 75 * 60000)) },
  { id: 5, user_id: 8, username: 'tester', loginType: '密码登录', ip: '203.0.113.99', location: '未知', success: false, status: 0, fail_reason: '密码错误', created_at: toEpochSeconds(new Date(nowMs - 90 * 60000)) },
  { id: 6, user_id: 6, username: 'support', loginType: '密码登录', ip: '192.0.2.30', location: '示例地区', success: true, status: 1, created_at: toEpochSeconds(new Date(nowMs - 120 * 60000)) },
  { id: 7, user_id: 1, username: 'admin', loginType: 'OAuth 单点登录', ip: '192.0.2.5', location: '本机', success: true, status: 1, created_at: toEpochSeconds(new Date(nowMs - 180 * 60000)) },
  { id: 8, user_id: 4, username: 'analyst', loginType: '密码登录', ip: '192.0.2.17', location: '示例地区', success: true, status: 1, created_at: toEpochSeconds(new Date(nowMs - 240 * 60000)) },
];

// ==================== 租户列表 (5 个) ====================

export const mockTenants = [
  { id: 1, name: '示例城市A', code: 'WS-OPS', plan: 'professional', status: 'active', domain: 'demo.example.com', user_count: 8, maxUsers: 200, created_at: '2026-01-05 10:00:00', updated_at: '2026-08-28 09:00:00' },
  { id: 2, name: '示例城市B', code: 'KM-OPS', plan: 'enterprise', status: 'active', domain: 'demo2.example.com', user_count: 6, maxUsers: 500, created_at: '2026-02-14 09:30:00', updated_at: '2026-08-27 18:22:00' },
  { id: 3, name: '示例城市C', code: 'DL-OPS', plan: 'basic', status: 'trial', domain: 'demo3.example.com', user_count: 3, maxUsers: 50, created_at: '2026-03-20 14:00:00', updated_at: '2026-08-26 11:05:00' },
  { id: 4, name: '测试租户', code: 'TEST-001', plan: 'free', status: 'disabled', domain: 'test.example.com', user_count: 2, maxUsers: 10, created_at: '2026-05-11 16:40:00', updated_at: '2026-08-20 10:00:00' },
  { id: 5, name: '平台租户', code: 'PLATFORM', plan: 'enterprise', status: 'active', domain: 'platform.example.com', user_count: 12, maxUsers: 1000, created_at: '2025-12-01 08:00:00', updated_at: '2026-08-28 08:10:00' },
];

// ==================== 文件列表 (7 个) ====================

export const mockFiles = [
  { id: 1, name: '示例运营方案.pdf', path: '/doc/示例运营方案.pdf', size: 2516582, mimeType: 'application/pdf', extension: 'pdf', isFolder: false, is_dir: false, isImage: false, type: 'file', created_by: { id: 1, username: 'admin' }, created_at: '2026-08-26 10:12:00', updated_at: '2026-08-26 10:12:00' },
  { id: 2, name: '车辆巡检视频_20260828.mp4', path: '/video/车辆巡检视频_20260828.mp4', size: 134217728, mimeType: 'video/mp4', extension: 'mp4', isFolder: false, is_dir: false, isImage: false, type: 'file', created_by: { id: 5, username: 'dispatcher' }, created_at: '2026-08-28 07:30:00', updated_at: '2026-08-28 07:30:00' },
  { id: 3, name: '示例停车点图片.png', path: '/img/示例停车点图片.png', size: 1258291, mimeType: 'image/png', extension: 'png', isFolder: false, is_dir: false, isImage: true, type: 'file', created_by: { id: 2, username: 'operator' }, created_at: '2026-08-27 15:45:00', updated_at: '2026-08-27 15:45:00' },
  { id: 4, name: '用户增长分析报告.docx', path: '/doc/用户增长分析报告.docx', size: 880640, mimeType: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document', extension: 'docx', isFolder: false, is_dir: false, isImage: false, type: 'file', created_by: { id: 4, username: 'analyst' }, created_at: '2026-08-25 18:20:00', updated_at: '2026-08-25 18:20:00' },
  { id: 5, name: '营收统计示例.xlsx', path: '/sheet/营收统计示例.xlsx', size: 524288, mimeType: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet', extension: 'xlsx', isFolder: false, is_dir: false, isImage: false, type: 'file', created_by: { id: 3, username: 'auditor' }, created_at: '2026-08-24 09:10:00', updated_at: '2026-08-24 09:10:00' },
  { id: 6, name: '排班表示例.xlsx', path: '/sheet/排班表示例.xlsx', size: 97280, mimeType: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet', extension: 'xlsx', isFolder: false, is_dir: false, isImage: false, type: 'file', created_by: { id: 2, username: 'operator' }, created_at: '2026-08-23 11:05:00', updated_at: '2026-08-23 11:05:00' },
  { id: 7, name: '平台操作手册.pdf', path: '/doc/平台操作手册.pdf', size: 5872025, mimeType: 'application/pdf', extension: 'pdf', isFolder: false, is_dir: false, isImage: false, type: 'file', created_by: { id: 1, username: 'admin' }, created_at: '2026-08-22 14:00:00', updated_at: '2026-08-22 14:00:00' },
];

// ==================== 文章列表 (6 个) ====================

export const mockArticles = [
  { id: 1, title: '关于规范停放的通告', categoryId: 2, categoryName: '运营公告', status: 'published', authorId: 1, authorName: '监管员', viewCount: 1280, likeCount: 32, commentCount: 8, shareCount: 5, isTop: true, isFeatured: false, isDraft: false, tags: ['停放', '规范'], publishedAt: '2026-08-25 09:00:00', created_at: '2026-08-24 16:20:00', updated_at: '2026-08-25 09:00:00' },
  { id: 2, title: '共享电单车骑行安全须知', categoryId: 2, categoryName: '运营公告', status: 'published', authorId: 5, authorName: '用户戊', viewCount: 2306, likeCount: 88, commentCount: 21, shareCount: 45, isTop: false, isFeatured: true, isDraft: false, tags: ['安全', '骑行'], publishedAt: '2026-08-23 10:30:00', created_at: '2026-08-22 14:00:00', updated_at: '2026-08-23 10:30:00' },
  { id: 3, title: '2026 年三季度车辆投放计划（草案）', categoryId: 3, categoryName: '技术文档', status: 'pending', authorId: 4, authorName: '用户丁', viewCount: 0, likeCount: 0, commentCount: 0, shareCount: 0, isTop: false, isFeatured: false, isDraft: false, tags: ['计划'], publishedAt: '', created_at: '2026-08-21 11:10:00', updated_at: '2026-08-21 11:10:00' },
  { id: 4, title: '节假日调度预案', categoryId: 1, categoryName: '政策法规', status: 'published', authorId: 2, authorName: '用户乙', viewCount: 864, likeCount: 20, commentCount: 6, shareCount: 12, isTop: false, isFeatured: false, isDraft: false, tags: ['调度', '预案'], publishedAt: '2026-08-20 08:00:00', created_at: '2026-08-19 17:30:00', updated_at: '2026-08-20 08:00:00' },
  { id: 5, title: '运维日报模板 v2（草稿）', categoryId: 3, categoryName: '技术文档', status: 'draft', authorId: 6, authorName: '用户己', viewCount: 0, likeCount: 0, commentCount: 0, shareCount: 0, isTop: false, isFeatured: false, isDraft: true, tags: ['模板'], publishedAt: '', created_at: '2026-08-18 09:45:00', updated_at: '2026-08-18 09:45:00' },
  { id: 6, title: '共享出行行业政策汇编（2026-08）', categoryId: 4, categoryName: '行业资讯', status: 'archived', authorId: 3, authorName: '用户丙', viewCount: 432, likeCount: 10, commentCount: 3, shareCount: 8, isTop: false, isFeatured: false, isDraft: false, tags: ['政策', '汇编'], publishedAt: '2026-08-15 12:00:00', created_at: '2026-08-14 10:00:00', updated_at: '2026-08-16 09:00:00' },
];

// ==================== 栏目列表 (4 个) ====================

export const mockCategories = [
  { id: 1, parent_id: 0, name: '政策法规', slug: 'policy', description: '政府与行业政策文件', sort_order: 1, status: 1, allowAttachment: false, created_at: '2026-01-10 09:00:00', updated_at: '2026-01-10 09:00:00' },
  { id: 2, parent_id: 0, name: '运营公告', slug: 'operation', description: '平台日常运营通知公告', sort_order: 2, status: 1, allowAttachment: true, created_at: '2026-01-10 09:05:00', updated_at: '2026-06-01 10:00:00' },
  { id: 3, parent_id: 0, name: '技术文档', slug: 'tech-docs', description: '技术方案与操作手册', sort_order: 3, status: 1, allowAttachment: true, created_at: '2026-02-02 14:00:00', updated_at: '2026-02-02 14:00:00' },
  { id: 4, parent_id: 0, name: '行业资讯', slug: 'news', description: '共享出行行业动态', sort_order: 4, status: 1, allowAttachment: false, created_at: '2026-03-15 11:30:00', updated_at: '2026-03-15 11:30:00' },
];

// ==================== 操作日志 (7 条) ====================

export const mockOperationLogs = [
  { id: 1, operator: 'admin', module: 'user', action: 'create', resource: '用户 #9', ip: '127.0.0.1', detail: '新增用户 tester2', created_at: toEpochSeconds(new Date(nowMs - 12 * 60000)) },
  { id: 2, operator: 'auditor', module: 'role', action: 'update', resource: '角色 #3', ip: '192.0.2.14', detail: '更新角色权限', created_at: toEpochSeconds(new Date(nowMs - 40 * 60000)) },
  { id: 3, operator: 'operator', module: 'department', action: 'update', resource: '部门 #2', ip: '192.0.2.8', detail: '调整部门成员', created_at: toEpochSeconds(new Date(nowMs - 68 * 60000)) },
  { id: 4, operator: 'admin', module: 'system', action: 'login', resource: '登录', ip: '127.0.0.1', detail: '后台登录成功', created_at: toEpochSeconds(new Date(nowMs - 95 * 60000)) },
  { id: 5, operator: 'dispatcher', module: 'ebike', action: 'update', resource: '车辆 MYK-0002', ip: '192.0.2.21', detail: '修改车辆停放区', created_at: toEpochSeconds(new Date(nowMs - 130 * 60000)) },
  { id: 6, operator: 'admin', module: 'system', action: 'delete', resource: '日志 #88', ip: '127.0.0.1', detail: '删除过期日志', created_at: toEpochSeconds(new Date(nowMs - 200 * 60000)) },
  { id: 7, operator: 'analyst', module: 'report', action: 'export', resource: '报表', ip: '192.0.2.17', detail: '导出月度统计报表', created_at: toEpochSeconds(new Date(nowMs - 300 * 60000)) },
];

// ==================== 消息列表 (6 条) ====================

export const mockNotifications = [
  { id: 1, user_id: 1, username: 'admin', title: '系统更新通知', content: '平台将于本周六 02:00-04:00 进行例行维护升级。', type: 'system', priority: 'normal', channel: 'in_app', status: 'read', is_read: true, read_at: '2026-08-28 08:00:00', created_at: '2026-08-27 20:00:00' },
  { id: 2, user_id: 2, username: 'operator', title: '新用户注册', content: '新用户「小李」已完成注册，请及时分配角色。', type: 'operation', priority: 'normal', channel: 'in_app', status: 'read', is_read: true, read_at: '2026-08-27 19:30:00', created_at: '2026-08-27 18:22:00' },
  { id: 3, user_id: 3, username: 'auditor', title: '登录异常提醒', content: '检测到您的账号在 192.0.2.99 登录失败 3 次。', type: 'system', priority: 'high', channel: 'in_app', status: 'sent', is_read: false, read_at: '', created_at: '2026-08-28 09:10:00' },
  { id: 4, user_id: 5, username: 'dispatcher', title: '审批提醒', content: '您有一笔车辆投放申请待审批。', type: 'approval', priority: 'high', channel: 'in_app', status: 'sent', is_read: false, read_at: '', created_at: '2026-08-28 08:45:00' },
  { id: 5, user_id: 1, username: 'admin', title: '数据备份完成', content: '昨日数据已备份完成，备份文件编号 20260827。', type: 'system', priority: 'low', channel: 'in_app', status: 'read', is_read: true, read_at: '2026-08-28 06:00:00', created_at: '2026-08-28 05:00:00' },
  { id: 6, user_id: 4, username: 'analyst', title: '月度报表已生成', content: '2026-08 运营月报已生成，请查收。', type: 'operation', priority: 'normal', channel: 'in_app', status: 'pending', is_read: false, read_at: '', created_at: '2026-08-28 07:00:00' },
];

// ==================== 流程列表 (6 个) ====================

export const mockWorkflows = [
  { id: 'wf-1001', name: '车辆投放审批流程', description: '新批次车辆投放的逐级审批', definition: {}, status: 'published', version: 3, created_by: 'admin', created_at: '2026-08-20 09:00:00', updated_at: '2026-08-22 15:30:00' },
  { id: 'wf-1002', name: '运维工单处理流程', description: '故障上报与派工处理闭环', definition: {}, status: 'published', version: 2, created_by: 'operator', created_at: '2026-08-18 10:20:00', updated_at: '2026-08-19 11:00:00' },
  { id: 'wf-1003', name: '城市开通申请流程', description: '新城市业务开通审批', definition: {}, status: 'draft', version: 1, created_by: 'auditor', created_at: '2026-08-16 14:00:00', updated_at: '2026-08-16 14:00:00' },
  { id: 'wf-1004', name: '供应商结算流程', description: '合作供应商月度结算', definition: {}, status: 'published', version: 5, created_by: 'admin', created_at: '2026-08-10 09:30:00', updated_at: '2026-08-15 16:40:00' },
  { id: 'wf-1005', name: '用户申诉处理流程', description: '用户订单申诉与退款', definition: {}, status: 'disabled', version: 2, created_by: 'support', created_at: '2026-08-05 11:00:00', updated_at: '2026-08-12 10:00:00' },
  { id: 'wf-1006', name: '节假日保障预案流程', description: '重大节假日运力保障', definition: {}, status: 'draft', version: 1, created_by: 'dispatcher', created_at: '2026-08-01 08:30:00', updated_at: '2026-08-01 08:30:00' },
];

// ==================== 车牌识别记录 (6 条) ====================

export const mockLprRecords = [
  { id: 1, plate_no: '云A·00001', plate_color: 'blue', plate_type: 'standard', vehicle_type: 'car', device_id: 'lpr-001', device_name: '示例相机A', park_code: 'WSC-01', lane_code: 'L1', direction: 'entry', pass_time: '2026-08-28 08:12:33', image_url: '', confidence: 0.98, status: 'processed', related_order_id: 'MYK-0001', remark: '', created_at: '2026-08-28 08:12:33', updated_at: '2026-08-28 08:12:35' },
  { id: 2, plate_no: '云A·00002', plate_color: 'blue', plate_type: 'standard', vehicle_type: 'car', device_id: 'lpr-001', device_name: '示例相机A', park_code: 'WSC-01', lane_code: 'L1', direction: 'exit', pass_time: '2026-08-28 07:50:11', image_url: '', confidence: 0.96, status: 'processed', related_order_id: '', remark: '', created_at: '2026-08-28 07:50:11', updated_at: '2026-08-28 07:50:13' },
  { id: 3, plate_no: '云A·00003', plate_color: 'green', plate_type: 'new_energy', vehicle_type: 'truck', device_id: 'lpr-002', device_name: '示例相机B', park_code: 'WSC-02', lane_code: 'L2', direction: 'entry', pass_time: '2026-08-28 07:31:05', image_url: '', confidence: 0.91, status: 'pending', related_order_id: '', remark: '等待人工复核', created_at: '2026-08-28 07:31:05', updated_at: '2026-08-28 07:31:05' },
  { id: 4, plate_no: '云A·00004', plate_color: 'yellow', plate_type: 'standard', vehicle_type: 'coach', device_id: 'lpr-003', device_name: '示例站点C相机', park_code: 'WSC-03', lane_code: 'L1', direction: 'entry', pass_time: '2026-08-28 06:45:58', image_url: '', confidence: 0.86, status: 'failed', related_order_id: '', remark: '识别失败，需补拍', created_at: '2026-08-28 06:45:58', updated_at: '2026-08-28 06:46:10' },
  { id: 5, plate_no: '云A·00005', plate_color: 'blue', plate_type: 'standard', vehicle_type: 'car', device_id: 'lpr-002', device_name: '示例相机B', park_code: 'WSC-02', lane_code: 'L2', direction: 'exit', pass_time: '2026-08-28 06:20:47', image_url: '', confidence: 0.97, status: 'processed', related_order_id: 'MYK-0004', remark: '', created_at: '2026-08-28 06:20:47', updated_at: '2026-08-28 06:20:49' },
  { id: 6, plate_no: '云A·00006', plate_color: 'blue', plate_type: 'standard', vehicle_type: 'suv', device_id: 'lpr-001', device_name: '示例相机A', park_code: 'WSC-01', lane_code: 'L1', direction: 'entry', pass_time: '2026-08-28 05:58:22', image_url: '', confidence: 0.94, status: 'processed', related_order_id: '', remark: '', created_at: '2026-08-28 05:58:22', updated_at: '2026-08-28 05:58:24' },
];

// ==================== 认证 ====================

const mockAdminUser = {
  id: 1,
  username: 'admin',
  nickname: '监管员',
  role: 'admin',
  status: 1,
  email: 'admin@example.com',
  phone: '13800000001',
  permissions: ['*'],
};

// ==================== Mock 路由 ====================

export function resolveMock(req: {
  path: string;
  method: string;
  body?: unknown;
  params?: Record<string, unknown>;
}): MockMatchResult | null {
  const path = (req.path || '').split('?')[0].replace(/^\/api/, '');
  const method = (req.method || 'GET').toUpperCase();
  const body = (req.body ?? {}) as Record<string, unknown>;

  // 1. 登录: POST 任何 /login
  if (method === 'POST' && path.endsWith('/login')) {
    return ok({
      token: `mock-token-${Date.now().toString(36)}`,
      refresh_token: `mock-refresh-${Date.now().toString(36)}`,
      user: { id: 1, username: 'admin', nickname: '监管员', role: 'admin' },
    });
  }

  // 2. 当前用户信息
  if (method === 'GET' && (path === '/user/info' || path.endsWith('/users/me'))) {
    return ok(mockAdminUser);
  }

  // 3. Token 刷新
  if (method === 'POST' && path.endsWith('/auth/refresh')) {
    return ok({ access_token: `mock-token-refreshed-${Date.now().toString(36)}`, refresh_token: 'mock-refresh-token' });
  }

  // 4. 电单车业务 (/v1/ebike)
  if (path.includes('/v1/ebike')) {
    if (method === 'POST' && path.endsWith('/options')) {
      return ok([{ level: 0, options: { system: 5, alert: 5 } }]);
    }

    if (method === 'GET' && path.includes('/info/')) {
      return ok({ msg: { id: path.split('/info/')[1] ?? '1', username: 'admin', password: '', level: 0, name: '监管员' } });
    }

    if (method === 'POST' && path.endsWith('/car')) {
      const op = asString(body.method);
      if (op === 'alert') {
        return ok(mockEbikes.filter((c) => c.alert !== ''));
      }
      if (op === 'history') {
        const code = asString(body.code);
        const target = mockEbikes.find((c) => c.code === code) ?? mockEbikes[0];
        const points: MockCar[] = [];
        const rand = mulberry32(code.length + 7);
        for (let i = 5; i >= 1; i--) {
          const backMs = i * 30 * 60000;
          points.push({
            ...(target as MockCar),
            gps: {
              lng: roundNum((target?.gps.lng ?? 104.0000) + (rand() - 0.5) * 0.01, 6),
              lat: roundNum((target?.gps.lat ?? 23.0000) + (rand() - 0.5) * 0.01, 6),
            },
            speed: roundNum(rand() * 25, 1),
            time: { start: fmtTime(new Date(nowMs - backMs)), end: fmtTime(new Date(nowMs - backMs + 300000)) },
            create_date: fmtTime(new Date(nowMs - backMs)),
          });
        }
        return ok(points);
      }
      // query: 按识别码/状态过滤 + 分页
      const code = asString(body.code);
      const status = Number(body.status ?? -1);
      const limit = Number(body.limit ?? 20);
      const offset = Number(body.offset ?? 0);
      let list = mockEbikes;
      if (code) list = list.filter((c) => c.code.toLowerCase().includes(code.toLowerCase()));
      if (status !== -1) list = list.filter((c) => c.status === status);
      return ok({ cars: list.slice(offset, offset + limit), total: list.length });
    }

    if (method === 'POST' && path.endsWith('/storage')) {
      return ok(mockStorages);
    }

    if (method === 'POST' && path.endsWith('/order')) {
      return ok(mockOrders);
    }
  }

  // 5. 首页 KPI
  if (method === 'GET' && path === '/admin/stats') {
    return ok(mockDashboardKpi);
  }

  // 6. 登录日志
  if (method === 'GET' && path === '/audit/login-logs') {
    return ok({ list: mockLoginLogs, total: mockLoginLogs.length });
  }

  // 7. 用户列表（含分页/搜索参数变体）
  if (method === 'GET' && path.includes('/admin/users') && !path.includes('batch') && !path.includes('export') && !path.includes('import')) {
    return ok({ list: mockUsers, total: mockUsers.length });
  }

  // 8. 租户列表
  if (method === 'GET' && path === '/admin/tenants') {
    return ok({ list: mockTenants, total: mockTenants.length });
  }

  // 9. 文件列表
  if (method === 'GET' && path === '/files') {
    return ok({ list: mockFiles, total: mockFiles.length, type: 'file' });
  }

  // 10. 文章列表
  if (method === 'GET' && path === '/cms/articles') {
    return ok({ list: mockArticles, total: mockArticles.length });
  }

  // 11. 栏目列表
  if (method === 'GET' && path === '/cms/categories') {
    return ok(mockCategories);
  }

  // 12. 操作日志
  if (method === 'GET' && path === '/audit/operation-logs') {
    return ok({ list: mockOperationLogs, total: mockOperationLogs.length });
  }

  // 13. 消息列表
  if (method === 'GET' && path === '/admin/notifications') {
    return ok({ list: mockNotifications, total: mockNotifications.length });
  }

  // 14. 流程列表
  if (method === 'GET' && path === '/workflows') {
    return ok({ list: mockWorkflows, total: mockWorkflows.length });
  }

  // 15. 车牌识别记录
  if (method === 'GET' && path === '/v1/lpr/records') {
    return ok({ list: mockLprRecords, total: mockLprRecords.length });
  }

  // 16. 角色权限 (admin 全放行)
  if (method === 'GET' && /^\/admin\/roles\/[^/]+\/permissions$/.test(path)) {
    return ok(['*']);
  }

  // 17. 兜底: 不阻塞渲染
  return ok(null);
}
