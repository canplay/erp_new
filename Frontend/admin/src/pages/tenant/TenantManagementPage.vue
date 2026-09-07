<template>
<q-page class="q-pa-md tenant-management">
    <!-- 页面标题 -->
    <div class="text-h5 q-mb-lg text-weight-bold">{{ $t('tenant.title') }}</div>

    <!-- 租户信息卡片 -->
    <q-card class="q-mb-md" bordered>
      <q-card-section>
        <div class="row items-center">
          <q-avatar size="60px" square>
            <img :src="tenantStore.currentTenant?.logo || '/logo.png'" />
          </q-avatar>
          <div class="q-ml-md">
            <div class="text-h6">{{ tenantStore.currentTenant?.name }}</div>
            <div class="text-caption text-grey">
              {{ tenantStore.currentTenant?.domain || tenantStore.currentTenant?.code }}
            </div>
          </div>
          <q-space />
          <q-badge
            :color="getPlanColor(tenantStore.currentTenant?.plan)"
            :label="getPlanLabel(tenantStore.currentTenant?.plan)"
          />
        </div>
      </q-card-section>
    </q-card>

    <!-- 使用统计 -->
    <div class="row q-col-gutter-md q-mb-md">
      <div class="col-12 col-sm-4">
        <q-card bordered>
          <q-card-section>
            <div class="text-caption text-grey">{{ $t('tenant.userQuota') }}</div>
            <div class="text-h5 q-mt-sm">
              {{ tenantStore.usageStats?.users?.used || 0 }} /
              {{ tenantStore.usageStats?.users?.limit || 0 }}
            </div>
            <q-linear-progress
              :value="tenantStore.userUsagePercent / 100"
              color="primary"
              class="q-mt-sm"
            />
          </q-card-section>
        </q-card>
      </div>
      <div class="col-12 col-sm-4">
        <q-card bordered>
          <q-card-section>
            <div class="text-caption text-grey">{{ $t('tenant.storageQuota') }}</div>
            <div class="text-h5 q-mt-sm">
              {{ formatBytes(tenantStore.usageStats?.storage?.used || 0) }} /
              {{ formatBytes(tenantStore.usageStats?.storage?.limit || 0) }}
            </div>
            <q-linear-progress
              :value="tenantStore.storageUsagePercent / 100"
              color="warning"
              class="q-mt-sm"
            />
          </q-card-section>
        </q-card>
      </div>
      <div class="col-12 col-sm-4">
        <q-card bordered>
          <q-card-section>
            <div class="text-caption text-grey">{{ $t('tenant.apiQuota') }}</div>
            <div class="text-h5 q-mt-sm">
              {{ formatNumber(tenantStore.usageStats?.apiCalls?.used || 0) }} /
              {{ formatNumber(tenantStore.usageStats?.apiCalls?.limit || 0) }}
            </div>
            <q-linear-progress
              :value="tenantStore.apiUsagePercent / 100"
              color="info"
              class="q-mt-sm"
            />
          </q-card-section>
        </q-card>
      </div>
    </div>

    <!-- 标签页 -->
    <q-tabs
      v-model="tab"
      class="q-mb-md"
      align="left"
      active-color="primary"
      indicator-color="primary"
    >
      <q-tab name="settings" :label="$t('tenant.settings')" icon="settings" />
      <q-tab name="members" :label="$t('tenant.members')" icon="people" />
      <q-tab name="billing" :label="$t('tenant.billing')" icon="receipt" />
    </q-tabs>

    <q-tab-panels v-model="tab" animated>
      <!-- 设置面板 -->
      <q-tab-panel name="settings">
        <q-card bordered>
          <q-card-section>
            <div class="text-h6 q-mb-md">{{ $t('tenant.settings') }}</div>
            <q-form class="q-gutter-md">
              <q-input
                v-model="tenantForm.name"
                :label="$t('tenant.name')"
                outlined
              />
              <q-input
                v-model="tenantForm.domain"
                :label="$t('tenant.domain')"
                outlined
                :placeholder="$t('tenant.domainPlaceholder')"
              />
              <div>
                <q-btn color="primary" :label="$t('common.save')" @click="handleSaveTenant" />
              </div>
            </q-form>
          </q-card-section>
        </q-card>
      </q-tab-panel>

      <!-- 成员面板 -->
      <q-tab-panel name="members">
        <q-card bordered>
          <q-card-section>
            <div class="row items-center q-mb-md">
              <div class="text-h6">{{ $t('tenant.members') }}</div>
              <q-space />
              <q-btn color="positive" :label="$t('tenant.addMember')" icon="add" @click="showAddMemberDialog = true" />
            </div>
            <q-table
              :rows="tenantStore.users"
              :columns="memberColumns"
              row-key="user_id"
              flat
              bordered
            >
              <template v-slot:body-cell-role="props">
                <q-td :props="props">
                  <q-chip :color="getRoleColor(props.value)" text-color="white" dense>
                    {{ getRoleLabel(props.value) }}
                  </q-chip>
                </q-td>
              </template>
              <template v-slot:body-cell-actions="props">
                <q-td :props="props">
                  <q-btn flat dense color="primary" :label="$t('common.edit')" @click="handleEditMember(props.row)" />
                  <q-btn flat dense color="negative" :label="$t('common.delete')" @click="handleRemoveMember(props.row)" />
                </q-td>
              </template>
            </q-table>
          </q-card-section>
        </q-card>
      </q-tab-panel>

      <!-- 套餐面板 -->
      <q-tab-panel name="billing">
        <div class="text-h6 q-mb-md">{{ $t('tenant.plans') }}</div>
        <div class="row q-col-gutter-md">
          <div v-for="plan in plans" :key="plan.id" class="col-12 col-sm-6 col-md-3">
            <q-card
              bordered
              :class="{ 'bg-primary text-white': plan.id === tenantStore.currentTenant?.plan }"
            >
              <q-card-section>
                <div class="text-h6">{{ plan.name }}</div>
                <div class="text-h4 q-my-sm">
                  ¥{{ plan.price }}<span class="text-caption">/{{ $t('tenant.' + plan.interval) }}</span>
                </div>
                <div class="text-caption q-mb-md">{{ plan.description }}</div>
                <q-list separator>
                  <q-item v-for="(feature, idx) in plan.features" :key="idx">
                    <q-item-section avatar>
                      <q-icon name="check" size="xs" />
                    </q-item-section>
                    <q-item-section>{{ feature }}</q-item-section>
                  </q-item>
                </q-list>
              </q-card-section>
              <q-card-actions vertical align="stretch">
                <q-btn
                  v-if="plan.id !== tenantStore.currentTenant?.plan"
                  color="primary"
                  :label="$t('tenant.upgrade')"
                  @click="handleUpgrade(plan)"
                />
                <q-btn
                  v-else
                  flat
                  disabled
                  :label="$t('tenant.current')"
                />
              </q-card-actions>
            </q-card>
          </div>
        </div>

        <!-- 升级确认对话框 -->
        <q-dialog v-model="showUpgradeDialog" persistent>
          <q-card style="min-width: 400px">
            <q-card-section>
              <div class="text-h6">{{ $t('tenant.upgradeConfirm') }}</div>
            </q-card-section>
            <q-card-section v-if="selectedPlan">
              <div class="q-mb-md">
                <div class="text-caption text-grey">{{ $t('tenant.targetPlan') }}</div>
                <div class="text-h6">{{ selectedPlan.name }}</div>
              </div>
              <div class="q-mb-md">
                <div class="text-caption text-grey">{{ $t('tenant.price') }}</div>
                <div class="text-h5 text-primary">¥{{ selectedPlan.price }}/{{ $t('tenant.' + selectedPlan.interval) }}</div>
              </div>
              <q-separator class="q-my-md" />
              <div class="text-caption text-grey">{{ $t('tenant.paymentMethod') }}</div>
              <q-option-group
                v-model="paymentInterval"
                :options="[
                  { label: $t('tenant.monthly'), value: 'month' },
                  { label: $t('tenant.yearly'), value: 'year' }
                ]"
                color="primary"
              />
            </q-card-section>
            <q-card-actions align="right">
              <q-btn flat :label="$t('common.cancel')" v-close-popup />
              <q-btn color="primary" :label="$t('tenant.confirmUpgrade')" @click="handleConfirmUpgrade" />
            </q-card-actions>
          </q-card>
        </q-dialog>
      </q-tab-panel>
    </q-tab-panels>

    <!-- 添加成员对话框 -->
    <q-dialog v-model="showAddMemberDialog">
      <q-card style="min-width: 400px">
        <q-card-section>
          <div class="text-h6">{{ $t('tenant.addMember') }}</div>
        </q-card-section>
        <q-card-section>
          <q-form class="q-gutter-md">
            <q-input
              v-model="memberForm.email"
              :label="$t('tenant.userEmail')"
              outlined
              type="email"
            />
            <q-select
              v-model="memberForm.role"
              :options="roleOptions"
              :label="$t('tenant.role')"
              outlined
              emit-value
              map-options
            />
          </q-form>
        </q-card-section>
        <q-card-actions align="right">
          <q-btn flat :label="$t('common.cancel')" v-close-popup />
          <q-btn color="primary" :label="$t('common.confirm')" @click="handleAddMember" />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useTenantManagement } from '@/composables/useTenantManagement';

const {
  tab, showAddMemberDialog, tenantForm, memberForm, roleOptions,
  showUpgradeDialog, selectedPlan, paymentInterval, plans, memberColumns,
  tenantStore, getPlanColor, getPlanLabel, getRoleColor, getRoleLabel,
  formatBytes, formatNumber, handleSaveTenant, handleAddMember,
  handleEditMember, handleRemoveMember, handleUpgrade, handleConfirmUpgrade,
  initData,
} = useTenantManagement();

onMounted(() => { void initData(); });
</script>

<style scoped>
.tenant-management { min-height: 100vh; }
</style>
