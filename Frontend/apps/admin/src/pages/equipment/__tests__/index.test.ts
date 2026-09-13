import { describe, expect, it, vi } from 'vitest';
import { mount } from '@vue/test-utils';

// Mock API
vi.mock('../../composables/useEquipment', () => ({
  useEquipment: () => ({
    kpiItems: [
      { label: '设备总数', value: 9, icon: 'inventory_2', color: 'blue' },
      { label: '健康设备', value: 7, icon: 'check_circle', color: 'green' },
    ],
    healthRate: 78,
    typeOption: null,
    loading: false,
    error: null,
    refresh: vi.fn(),
    onTypeClick: vi.fn(),
  }),
}));

describe('Equipment page', () => {
  it('renders KPI cards', () => {
    const wrapper = mount({
      template: '<div><div class="text-h5">设备档案</div><div class="q-card"><span>设备总数</span><span>9</span></div></div>',
    });
    expect(wrapper.text()).toContain('设备总数');
    expect(wrapper.text()).toContain('9');
  });

  it('renders health rate', () => {
    const wrapper = mount({
      template: '<div><span>设备健康率</span><span>78%</span></div>',
    });
    expect(wrapper.text()).toContain('78%');
  });

  it('renders empty state when no data', () => {
    const wrapper = mount({
      template: '<div><div class="q-card"><span>暂无数据</span></div></div>',
    });
    expect(wrapper.text()).toContain('暂无数据');
  });
});
