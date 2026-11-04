import { Card } from '@/components/ui/card';
import { formatNumber, formatPercent } from '@/lib/format';
import type { PlayerDetailPayload } from '@cngokz/shared-types';

export function OverallStats({ player }: { player: PlayerDetailPayload }) {
  // The page explicitly shows the selected scope so all cards below the header
  // agree with tier progress and map completion on the same page.
  const overall = player.scoped;
  const stats = [
    {
      label: '完成地图',
      value: formatNumber(overall.completed_maps),
      sub: `共 ${formatNumber(overall.total_maps)} 张`,
    },
    { label: '地图完成率', value: formatPercent(overall.completion_rate), sub: '主线地图' },
    { label: '世界纪录', value: formatNumber(overall.wr_count), sub: '当前 WR 指针' },
    {
      label: '总成绩数',
      value: formatNumber(overall.total_records),
      sub: `PRO ${overall.pro_count} / NUB ${overall.tp_count}`,
    },
    { label: 'PRO 成绩', value: formatNumber(overall.pro_count), sub: '0 传送' },
    { label: 'TP 成绩', value: formatNumber(overall.tp_count), sub: '含传送' },
    { label: '全球完成排名', value: overall.global_rank ? `#${overall.global_rank}` : '—', sub: '按完成地图数' },
    { label: '积分 / Rating', value: '—', sub: '积分算法未定义' },
  ];

  return (
    <div className="space-y-2">
      <div className="flex flex-wrap items-center gap-2 text-[11px] text-muted-foreground">
        <span className="font-medium text-foreground">当前范围</span>
        <span className="font-mono">
          {player.scope.mode.display_name} · {player.scope.style.display_name} · {player.scope.time_type.toUpperCase()}
        </span>
        <span>· 全站汇总见下方模式卡</span>
      </div>
      <div className="grid grid-cols-2 gap-3 lg:grid-cols-4">
        {stats.map((stat) => (
          <Card key={stat.label} className="gap-1 p-4">
            <p className="text-[11px] text-muted-foreground">{stat.label}</p>
            <p className="text-xl font-semibold tracking-tight tabular-nums">{stat.value}</p>
            <p className="text-[10px] text-muted-foreground">{stat.sub}</p>
          </Card>
        ))}
      </div>
    </div>
  );
}
