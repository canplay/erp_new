/**
 * @file useTenantManagement.ts
 * @description 租户管理页面业务逻辑 composable
 */

import { ref, reactive } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { useTenantStore } from '@/stores/tenant';
import type { TenantUser } from '@/types/tenant';
import { upgradePlan, searchUsersByEmail } from '@/api/tenant';
import { logger } from '@/utils/logger';

export function useTenantManagement() {
  const $q = useQuasar();
  const { t: $t } = useI18n();
  const tenantStore = useTenantStore();

  const tab = ref('settings');
  const showAddMemberDialog = ref(false);
  const tenantForm = reactive({ name: '', domain: '' });
  const memberForm = reactive({ email: '', role: 'member' });

  const roleOptions = [
    { label: '所有者', value: 'owner' },
    { label: '管理员', value: 'admin' },
    { label: '成员', value: 'member' },
    { label: '访客', value: 'guest' },
  ];

  const showUpgradeDialog = ref(false);
  const selectedPlan = ref<{ id: string; name: string; price: number; interval: string } | null>(null);
  const paymentInterval = ref<'month' | 'year'>('year');

  interface Plan {
    id: string;
    name: string;
    price: number;
    interval: string;
    description: string;
    features: string[];
  }

  const plans: Plan[] = [
    { id: 'free', name: '免费版', price: 0, interval: 'month', description: '适合个人或小团队入门使用', features: ['5个用户', '1GB存储', '1000次API调用/天', '基础支持'] },
    { id: 'basic', name: '基础版', price: 99, interval: 'month', description: '适合中小团队日常使用', features: ['20个用户', '50GB存储', '10000次API调用/天', '邮件支持', '优先响应'] },
    { id: 'professional', name: '专业版', price: 299, interval: 'month', description: '适合成长中的企业团队', features: ['100个用户', '200GB存储', '不限API调用', '7x24支持', '高级分析', '自定义Logo'] },
    { id: 'enterprise', name: '企业版', price: 999, interval: 'month', description: '适合大型企业或特殊需求', features: ['无限用户', '无限存储', '不限API调用', '专属客服', '私有部署', 'SLA保障'] },
  ];

  interface MemberColumn {
    name: string;
    label: string;
    field: string;
    align: 'left' | 'center' | 'right';
  }

  const memberColumns: MemberColumn[] = [
    { name: 'username', label: $t('user.username'), field: 'username', align: 'left' },
    { name: 'email', label: $t('user.email'), field: 'email', align: 'left' },
    { name: 'role', label: $t('tenant.role'), field: 'role', align: 'center' },
    { name: 'joinedAt', label: $t('tenant.joinedAt'), field: 'joinedAt', align: 'left' },
    { name: 'actions', label: $t('common.actions'), field: 'actions', align: 'center' },
  ];

  function getPlanColor(plan?: string): string {
    const colors: Record<string, string> = { free: 'grey', basic: 'blue', professional: 'purple', enterprise: 'warning' };
    return colors[plan || 'free'] || 'grey';
  }

  function getPlanLabel(plan?: string): string {
    const labels: Record<string, string> = { free: '免费版', basic: '基础版', professional: '专业版', enterprise: '企业版' };
    return labels[plan || 'free'] || plan || '';
  }

  function getRoleColor(role: string): string {
    const colors: Record<string, string> = { owner: 'warning', admin: 'primary', member: 'positive', guest: 'grey' };
    return colors[role] || 'grey';
  }

  function getRoleLabel(role: string): string {
    const labels: Record<string, string> = { owner: '所有者', admin: '管理员', member: '成员', guest: '访客' };
    return labels[role] || role;
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
  }

  function formatNumber(num: number): string {
    if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M';
    if (num >= 1000) return (num / 1000).toFixed(1) + 'K';
    return num.toLocaleString();
  }

  async function handleSaveTenant() {
    try {
      await tenantStore.saveTenant(tenantForm);
      $q.notify({ type: 'positive', message: $t('common.success') });
    } catch {
      $q.notify({ type: 'negative', message: $t('common.error') });
    }
  }

  async function handleAddMember() {
    if (!memberForm.email) { $q.notify({ type: 'warning', message: '请输入邮箱地址' }); return; }
    try {
      const response = await searchUsersByEmail(memberForm.email);
      const respData = response as { data?: { data?: { user_id: string }[] } };
      const userList = respData.data?.data;
      if (userList && userList.length > 0) {
        const user = userList[0]!;
        logger.info('【通过邮箱找到用户】', { user_id: user.user_id, email: memberForm.email });
        await tenantStore.addUser({ user_id: Number(user.user_id), role: memberForm.role });
        $q.notify({ type: 'positive', message: $t('common.success') });
        showAddMemberDialog.value = false;
        Object.assign(memberForm, { email: '', role: 'member' });
        await tenantStore.loadUsers();
      } else {
        $q.notify({ type: 'warning', message: '未找到该邮箱对应的用户' });
      }
    } catch (error) {
      logger.error('【添加租户成员失败】', error);
      $q.notify({ type: 'negative', message: $t('common.error') });
    }
  }

  function handleEditMember(member: TenantUser) { void member; }

  function handleRemoveMember(member: TenantUser) {
    $q.dialog({
      title: $t('common.confirm'),
      message: $t('tenant.removeMemberConfirm', { name: member.username }),
      cancel: true, persistent: true,
    }).onOk(() => {
      void (async () => {
        try { await tenantStore.removeUser(member.user_id); $q.notify({ type: 'positive', message: $t('common.success') }); }
        catch { $q.notify({ type: 'negative', message: $t('common.error') }); }
      })();
    });
  }

  function handleUpgrade(plan: Plan) { selectedPlan.value = plan; showUpgradeDialog.value = true; }

  async function handleConfirmUpgrade() {
    if (!selectedPlan.value) return;
    try {
      await upgradePlan(selectedPlan.value.id, paymentInterval.value);
      $q.notify({ type: 'positive', message: $t('tenant.upgradeSuccess') });
      showUpgradeDialog.value = false;
      await tenantStore.loadCurrentTenant();
    } catch {
      $q.notify({ type: 'negative', message: $t('common.error') });
    }
  }

  async function initData() {
    await tenantStore.loadCurrentTenant();
    await tenantStore.loadUsageStats();
    await tenantStore.loadUsers();
    if (tenantStore.currentTenant) {
      Object.assign(tenantForm, { name: tenantStore.currentTenant.name, domain: tenantStore.currentTenant.domain || '' });
    }
  }

  return {
    tab, showAddMemberDialog, tenantForm, memberForm, roleOptions,
    showUpgradeDialog, selectedPlan, paymentInterval, plans, memberColumns,
    tenantStore, getPlanColor, getPlanLabel, getRoleColor, getRoleLabel,
    formatBytes, formatNumber, handleSaveTenant, handleAddMember,
    handleEditMember, handleRemoveMember, handleUpgrade, handleConfirmUpgrade,
    initData,
  };
}
