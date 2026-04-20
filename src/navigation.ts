export type AppSection = "overview" | "members" | "gifts" | "migration" | "settings" | "logs";

export type SectionOption = {
  key: AppSection;
  label: string;
  caption: string;
  description: string;
  to: string;
};

export const sectionOptions: SectionOption[] = [
  {
    key: "overview",
    label: "经营总览",
    caption: "Overview",
    description: "查看核心指标、月度排行与系统快照",
    to: "/overview"
  },
  {
    key: "members",
    label: "会员中心",
    caption: "Members",
    description: "会员检索、资料维护、消费与积分记录",
    to: "/members"
  },
  {
    key: "gifts",
    label: "礼品中心",
    caption: "Gifts",
    description: "浏览礼品库存并维护兑换配置",
    to: "/gifts"
  },
  {
    key: "migration",
    label: "数据迁移",
    caption: "Migration",
    description: "执行预检、导入批次并查看迁移结果",
    to: "/migration"
  },
  {
    key: "settings",
    label: "系统设置",
    caption: "Settings",
    description: "门店参数、备份目录与恢复操作",
    to: "/settings"
  },
  {
    key: "logs",
    label: "操作日志",
    caption: "Logs",
    description: "审计最近业务动作与异常信息",
    to: "/logs"
  }
];
