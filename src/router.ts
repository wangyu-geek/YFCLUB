import { createRouter, createWebHashHistory } from "vue-router";
import GiftsView from "./views/GiftsView.vue";
import LogsView from "./views/LogsView.vue";
import MembersView from "./views/MembersView.vue";
import MigrationView from "./views/MigrationView.vue";
import OverviewView from "./views/OverviewView.vue";
import SettingsView from "./views/SettingsView.vue";

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: "/",
      redirect: "/overview"
    },
    {
      path: "/overview",
      name: "overview",
      component: OverviewView
    },
    {
      path: "/members",
      name: "members",
      component: MembersView
    },
    {
      path: "/gifts",
      name: "gifts",
      component: GiftsView
    },
    {
      path: "/migration",
      name: "migration",
      component: MigrationView
    },
    {
      path: "/settings",
      name: "settings",
      component: SettingsView
    },
    {
      path: "/logs",
      name: "logs",
      component: LogsView
    },
    {
      path: "/:pathMatch(.*)*",
      redirect: "/overview"
    }
  ]
});

export default router;
