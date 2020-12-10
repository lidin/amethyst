use crate::{
    ecs::*
};

use crate::HiddenPropagate;
use crate::transform::{Children, Parent};
use std::collections::HashSet;

/// This system adds a [HiddenPropagate](struct.HiddenPropagate.html)-component to all children
/// of an entity with a [HiddenPropagate](struct.HiddenPropagate.html) and removes it when it is removed
/// from the parent.
pub fn build() -> impl Runnable {
    SystemBuilder::new("HideHierarchySystem")
        .with_query(<(&Children, Option<&HiddenPropagate>)>::query())
        .with_query(<(Entity, &Parent, Option<&HiddenPropagate>)>::query())
        .write_component::<HiddenPropagate>()
        .build(move |commands, world, _resources, (parent, children)| {
            #[cfg(feature = "profiler")]
        	profile_scope!("hide_hierarchy_system");

            let mut children_with_hidden_parent: HashSet<&Entity> = HashSet::new();
            let mut children_without_hidden_parent: HashSet<&Entity> = HashSet::new();

            for (current_children, hidden) in parent.iter(world) {
                if let Some(hidden_propagate) = hidden {
                    if hidden_propagate.is_propagated() {
                        current_children.0.iter().for_each(|e| { children_with_hidden_parent.insert(e); });
                    }
                } else {
                    current_children.0.iter().for_each(|e| { children_without_hidden_parent.insert(e); });
                }
            }

            for (entity, _, hidden) in children.iter(world){
                if let Some(hidden_propagate) = hidden {
                    if children_with_hidden_parent.contains(&entity) {
                        children_with_hidden_parent.remove(&entity);
                    }else if !hidden_propagate.is_propagated() && children_without_hidden_parent.contains(&entity){
                        children_without_hidden_parent.remove(&entity);
                    }
                }else if children_without_hidden_parent.contains(&entity){
                    children_without_hidden_parent.remove(&entity);
                }
            }
            children_with_hidden_parent.iter().for_each(|e| commands.add_component(**e, HiddenPropagate::new_propagated()));
            children_without_hidden_parent.iter().for_each(|e| commands.remove_component::<HiddenPropagate>(**e));
        })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::transform::{Transform, Parent, parent_update_system, missing_previous_parent_system};

    #[test]
    fn should_not_add_hidden_to_child_if_not_propagated() {
        let mut resources = Resources::default();
        let mut world = World::default();

        let mut schedule = Schedule::builder()
            .add_system(missing_previous_parent_system::build())
            .add_system(parent_update_system::build())
            .add_system(build())
            .build();

        let parent = world.push((Transform::default(), ));
        let children = world.extend(vec![(Transform::default(), ), (Transform::default(), )]);
        let (e1, e2) = (children[0], children[1]);
        // Parent `e1` and `e2` to `parent`.
        world.entry(e1).unwrap().add_component(Parent(parent));
        world.entry(e2).unwrap().add_component(Parent(parent));

        schedule.execute(&mut world, &mut resources);

        assert_eq!(
            true,
            world
                .entry(parent)
                .unwrap()
                .get_component::<HiddenPropagate>()
                .is_err()
        );

        assert_eq!(
            true,
            world
                .entry(e1)
                .unwrap()
                .get_component::<HiddenPropagate>()
                .is_err()
        );

        assert_eq!(
            true,
            world
                .entry(e2)
                .unwrap()
                .get_component::<HiddenPropagate>()
                .is_err()
        );

        world.entry(parent).unwrap().add_component(HiddenPropagate::new());

        schedule.execute(&mut world, &mut resources);

        assert_eq!(
            true,
            world
                .entry(parent)
                .unwrap()
                .get_component::<HiddenPropagate>()
                .is_ok()
        );

        schedule.execute(&mut world, &mut resources);

        assert_eq!(
            true,
            world
                .entry(e1)
                .unwrap()
                .get_component::<HiddenPropagate>()
                .is_err()
        );

        assert_eq!(
            true,
            world
                .entry(e2)
                .unwrap()
                .get_component::<HiddenPropagate>()
                .is_err()
        );
    }

    #[test]
    fn should_not_delete_hidden_from_child_if_not_propagated() {
        let mut resources = Resources::default();
        let mut world = World::default();

        let mut schedule = Schedule::builder()
            .add_system(missing_previous_parent_system::build())
            .add_system(parent_update_system::build())
            .add_system(build())
            .build();

        let parent = world.push((Transform::default(), ));
        let children = world.extend(vec![(Transform::default(), )]);
        let e1 = children[0];
        // Parent `e1` and `e2` to `parent`.
        world.entry(e1).unwrap().add_component(Parent(parent));
        world.entry(e1).unwrap().add_component(HiddenPropagate::new());

        schedule.execute(&mut world, &mut resources);

        assert_eq!(
            true,
            world
                .entry(parent)
                .unwrap()
                .get_component::<HiddenPropagate>()
                .is_err()
        );

        assert_eq!(
            true,
            world
                .entry(e1)
                .unwrap()
                .get_component::<HiddenPropagate>()
                .is_ok()
        );

        world.entry(parent).unwrap().add_component(HiddenPropagate::new());
        schedule.execute(&mut world, &mut resources);

        assert_eq!(
            true,
            world
                .entry(parent)
                .unwrap()
                .get_component::<HiddenPropagate>()
                .is_ok()
        );

        assert_eq!(
            true,
            world
                .entry(e1)
                .unwrap()
                .get_component::<HiddenPropagate>()
                .is_ok()
        );

        world.entry(parent).unwrap().remove_component::<HiddenPropagate>();
        schedule.execute(&mut world, &mut resources);

        assert_eq!(
            true,
            world
                .entry(parent)
                .unwrap()
                .get_component::<HiddenPropagate>()
                .is_err()
        );

        assert_eq!(
            true,
            world
                .entry(e1)
                .unwrap()
                .get_component::<HiddenPropagate>()
                .is_ok()
        );
    }

    #[test]
    fn should_add_and_delete_hidden_if_is_propagated() {
        let mut resources = Resources::default();
        let mut world = World::default();

        let mut schedule = Schedule::builder()
            .add_system(missing_previous_parent_system::build())
            .add_system(parent_update_system::build())
            .add_system(build())
            .build();

        let parent = world.push((Transform::default(), ));
        let children = world.extend(vec![(Transform::default(), ), (Transform::default(), )]);
        let (e1, e2) = (children[0], children[1]);
        // Parent `e1` and `e2` to `parent`.
        world.entry(e1).unwrap().add_component(Parent(parent));
        world.entry(e2).unwrap().add_component(Parent(parent));

        schedule.execute(&mut world, &mut resources);

        assert_eq!(
            true,
            world
                .entry(parent)
                .unwrap()
                .get_component::<HiddenPropagate>()
                .is_err()
        );

        assert_eq!(
            true,
            world
                .entry(e1)
                .unwrap()
                .get_component::<HiddenPropagate>()
                .is_err()
        );

        assert_eq!(
            true,
            world
                .entry(e2)
                .unwrap()
                .get_component::<HiddenPropagate>()
                .is_err()
        );

        world.entry(parent).unwrap().add_component(HiddenPropagate{is_propagated: true});

        schedule.execute(&mut world, &mut resources);

        assert_eq!(
            true,
            world
                .entry(parent)
                .unwrap()
                .get_component::<HiddenPropagate>()
                .is_ok()
        );

        schedule.execute(&mut world, &mut resources);

        assert_eq!(
            true,
            world
                .entry(e1)
                .unwrap()
                .get_component::<HiddenPropagate>()
                .is_ok()
        );

        assert_eq!(
            true,
            world
                .entry(e2)
                .unwrap()
                .get_component::<HiddenPropagate>()
                .is_ok()
        );

        world.entry(parent).unwrap().remove_component::<HiddenPropagate>();

        assert_eq!(
            true,
            world
                .entry(parent)
                .unwrap()
                .get_component::<HiddenPropagate>()
                .is_err()
        );

        schedule.execute(&mut world, &mut resources);

        assert_eq!(
            true,
            world
                .entry(e1)
                .unwrap()
                .get_component::<HiddenPropagate>()
                .is_err()
        );

        assert_eq!(
            true,
            world
                .entry(e2)
                .unwrap()
                .get_component::<HiddenPropagate>()
                .is_err()
        );
    }
}
