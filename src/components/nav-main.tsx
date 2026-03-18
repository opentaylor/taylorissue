import { NavLink, useLocation } from "react-router"
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible"
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/components/ui/sidebar"

interface NavItem {
  title: string
  url: string
  icon: React.ReactNode
  className?: string
  items?: { title: string; url: string }[]
}

export function NavMain({ items }: { items: NavItem[] }) {
  const { pathname } = useLocation()

  return (
    <SidebarMenu>
      {items.map((item) =>
        item.items ? (
          <CollapsibleMenuItem
            key={item.title}
            item={item}
            pathname={pathname}
          />
        ) : (
          <SidebarMenuItem key={item.title}>
            <NavLink to={item.url} end>
              {({ isActive }) => (
                <SidebarMenuButton isActive={isActive} className={item.className}>
                  {item.icon}
                  <span>{item.title}</span>
                </SidebarMenuButton>
              )}
            </NavLink>
          </SidebarMenuItem>
        ),
      )}
    </SidebarMenu>
  )
}

function CollapsibleMenuItem({
  item,
  pathname,
}: {
  item: NavItem
  pathname: string
}) {
  const isChildActive = item.items?.some((sub) => pathname === sub.url) ?? false

  return (
    <Collapsible defaultOpen={isChildActive} className="group/collapsible">
      <SidebarMenuItem>
        <CollapsibleTrigger className="w-full">
          <SidebarMenuButton isActive={isChildActive}>
            {item.icon}
            <span>{item.title}</span>
          </SidebarMenuButton>
        </CollapsibleTrigger>
        <CollapsibleContent>
          <SidebarMenuSub>
            {item.items?.map((sub) => (
              <SidebarMenuSubItem key={sub.url}>
                <NavLink to={sub.url}>
                  {({ isActive }) => (
                    <SidebarMenuSubButton
                      isActive={isActive}
                      render={<span />}
                    >
                      <span>{sub.title}</span>
                    </SidebarMenuSubButton>
                  )}
                </NavLink>
              </SidebarMenuSubItem>
            ))}
          </SidebarMenuSub>
        </CollapsibleContent>
      </SidebarMenuItem>
    </Collapsible>
  )
}
