use async_trait::async_trait;
use uuid::Uuid;

use crate::{error::ForumResult, models::{ChatPost, Post, Section, Thread, UserRole}};

#[async_trait]
pub trait ForumExt {
    async fn create_thread(&self, user: Uuid, section: i64, title: &str, content: &str, hash_tags: &[String]) -> ForumResult<()>;
    async fn delete_thread(&self, thread_id: i64) -> ForumResult<()>;
    async fn update_thread(&self, thread_id: i64, title: &str, content: &str) -> ForumResult<()>;
    async fn lock_thread(&self, thread_id: i64, locked: bool) -> ForumResult<()>;

    async fn create_section(&self, name: &str, description: &str, allowed_for: &[UserRole]) -> ForumResult<()>;
    async fn get_sections(&self, user: Option<Uuid>) -> ForumResult<Vec<Section>>;
    async fn delete_section(&self, s_id: i32) -> ForumResult<()>;

    async fn get_chat(&self, limit: usize) -> ForumResult<Vec<ChatPost>>;
    async fn post_chat(&self, u_id: Uuid, content: &str) -> ForumResult<()>;
    async fn delete_chat(&self, post_id: i32) -> ForumResult<()>;

    async fn get_section(&self, s_id: i64, page: i32, limit: usize) -> ForumResult<Vec<crate::dto::forum::ThreadListItemDto>>;
    async fn get_thread(&self, t_id: i64, page: i32, limit: usize) -> ForumResult<Vec<Post>>;
    async fn get_thread_info(&self, t_id: i32) -> ForumResult<Thread>;
    async fn get_thread_author(&self, t_id: i32) -> ForumResult<Uuid>;
    async fn get_thread_reply_count(&self, t_id: i32) -> ForumResult<i64>;

    async fn add_post(&self, author: Uuid, th_id: i64, content: &str, post_id: Option<i64>) -> ForumResult<()>;
    async fn update_post(&self, p_id: i64, content: &str) -> ForumResult<()>;
    async fn delete_post(&self, post_id: i64) -> ForumResult<()>;
    async fn get_post_author(&self, t_id: i64) -> ForumResult<Option<Uuid>>;
    async fn posts_since(&self, post_id: i64) -> ForumResult<i64>;
}

#[async_trait]
impl ForumExt for crate::db::DBClient {
    async fn create_thread(&self, user: Uuid, section: i64, title: &str, content: &str, hash_tags: &[String]) -> ForumResult<()> {
        struct ParsingHelper {
            id: i64,
        }

        let r = sqlx::query_as!(ParsingHelper, r#"INSERT INTO forum.threads(title,created_at,content,author,section,locked)
            VALUES ($1,LOCALTIMESTAMP,$2,$3,$4,false)
            RETURNING id"#,
            title, content, user, section)
            .fetch_one(&self.pool)
            .await?;
            let _ = hash_tags.iter().map(async |t| {
                sqlx::query!(r#"INSERT INTO forum.hashtags(tag, topic) VALUES($1, $2)"#, t, r.id)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| e)
            });
        Ok(())
    }

    async fn delete_thread(&self, thread_id: i64) -> ForumResult<()> {
        sqlx::query!(r#"DELETE FROM forum.threads WHERE id = $1"#, thread_id as i32)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_thread(&self, thread_id: i64, title: &str, content: &str) -> ForumResult<()> {
        sqlx::query!(r#"UPDATE forum.threads
            SET
                title = $2,
                content = $3
            WHERE id = $1"#, thread_id as i32, title, content)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn lock_thread(&self, thread_id: i64, locked: bool) -> ForumResult<()> {
        sqlx::query!(
            r#"UPDATE forum.threads
            SET locked = $2
            WHERE id = $1
            "#,
            thread_id as i32, locked)
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }

    async fn create_section(&self, name: &str, description: &str, allowed_for: &[UserRole]) -> ForumResult<()> {
        struct Helper {
            id: i64,
        }

        let r = sqlx::query_as!(Helper,
            r#" INSERT INTO forum.sections
                    (name, description)
                VALUES($1, $2)
                RETURNING id"#, name, description)
            .fetch_one(&self.pool)
            .await?;

        let _ = allowed_for.iter().map(async |rl| {
            sqlx::query!(r#"INSERT INTO forum.sections_allowed
                                (section, role)
                            VALUES($1, $2)"#, r.id, *rl as UserRole)
                .execute(&self.pool)
                .await
                .map_err(|e| e)
        });

        Ok(())
    }

    async fn get_sections(&self, user: Option<Uuid>) -> ForumResult<Vec<Section>> {
        match user {
            Some(user_id) => {
                let r = sqlx::query_as!(Section,
                    r#" SELECT s.id, s.name, s.description,
                        COALESCE(
                            CASE
                                WHEN u.last_online IS NOT NULL AND EXISTS (
                                    SELECT 1
                                    FROM forum.posts p
                                    INNER JOIN forum.threads t ON p.topic = t.id
                                    WHERE t.section = s.id
                                    AND p.created_at > u.last_online
                                ) THEN true
                                ELSE false
                            END, false
                        ) as "new_posts!: bool"
                        FROM forum.sections s
                        CROSS JOIN forum.users u
                        WHERE u.id = $1
                    "#, user_id)
                    .fetch_all(&self.pool)
                    .await?;
                Ok(r)
            }
            None => {
                // For anonymous users, return false for new_posts for all sections
                let r = sqlx::query_as!(Section,
                    r#" SELECT s.id, s.name, s.description,
                        false as "new_posts!: bool"
                        FROM forum.sections s
                    "#)
                    .fetch_all(&self.pool)
                    .await?;
                Ok(r)
            }
        }
    }

    async fn delete_section(&self, s_id: i32) -> ForumResult<()> {
        sqlx::query!(
            r#"DELETE FROM forum.sections
               WHERE id = $1"#, s_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_chat(&self, limit: usize) -> ForumResult<Vec<ChatPost>> {
        let limit = limit as i64;
        let r = sqlx::query_as!(ChatPost,
            r#" SELECT p.id,added,author,u.name as author_name,content FROM forum.chat_posts p
                INNER JOIN forum.users u ON author = u.id
                ORDER BY added DESC
                LIMIT $1"#, limit)
            .fetch_all(&self.pool)
            .await?;
        Ok(r)
    }

    async fn post_chat(&self, u_id: Uuid, content: &str) -> ForumResult<()> {
        sqlx::query!(
            r#" INSERT INTO forum.chat_posts(added, author, content)
                VALUES (LOCALTIMESTAMP, $1, $2)"#, u_id, content)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn delete_chat(&self, post_id: i32) -> ForumResult<()> {
        sqlx::query!(
            r#"DELETE FROM forum.chat_posts WHERE id = $1"#, post_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_section(&self, s_id: i64, page: i32, limit: usize) -> ForumResult<Vec<crate::dto::forum::ThreadListItemDto>> {
        let offset = (page - 1) as usize * limit;
        let limit = limit as i64;
        let offset = offset as i64;

        let r = sqlx::query_as!(crate::dto::forum::ThreadListItemDto,
            r#" SELECT t.id,t.title,t.created_at,t.content,t.author,u.name author_name,t.section as section_id,t.locked,t.sticky FROM forum.threads t
                INNER JOIN forum.users u ON t.author = u.id
                WHERE section = $1
                ORDER BY t.id DESC
                LIMIT $2 OFFSET $3"#, s_id, limit, offset)
            .fetch_all(&self.pool)
            .await?;
        Ok(r)
    }

    async fn get_thread(&self, t_id: i64, page: i32, limit: usize) -> ForumResult<Vec<Post>> {
        let offset = (page - 1) as usize * limit;
        let limit = limit as i64;
        let offset = offset as i64;
        let r = sqlx::query_as!(Post,
            r#" SELECT p.id,p.content,p.author,u.name as author_name,p.topic,p.comments,p.created_at,p.modified_at,p.likes 
                FROM forum.posts p
                LEFT OUTER JOIN forum.users u ON u.id = p.author
                WHERE topic = $1
                ORDER BY p.created_at ASC
                LIMIT $2 OFFSET $3"#, t_id, limit, offset)
            .fetch_all(&self.pool)
            .await?;
        Ok(r)
    }

    async fn get_thread_info(&self, t_id: i32) -> ForumResult<Thread> {
        let r = sqlx::query_as!(Thread,
            r#" SELECT * FROM forum.threads WHERE id = $1"#, t_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(r)
    }

    async fn get_thread_author(&self, t_id: i32) -> ForumResult<Uuid> {
        struct Helper {
            author: Uuid,
        }

        let res = sqlx::query_as!(Helper,
            r#"SELECT author FROM forum.threads WHERE id = $1"#, t_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(res.author)
    }

    async fn get_post_author(&self, t_id: i64) -> ForumResult<Option<Uuid>> {
        struct Helper {
            author: Option<Uuid>,
        }

        let res = sqlx::query_as!(Helper,
            r#"SELECT author FROM forum.posts WHERE id = $1"#, t_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(res.author)
    }


    async fn get_thread_reply_count(&self, t_id: i32) -> ForumResult<i64> {
        struct Helper {
            cnt: Option<i64>,
        }

        let res = sqlx::query_as!(Helper,
            r#"SELECT COUNT(*) cnt FROM forum.threads WHERE id = $1"#, t_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(res.cnt.unwrap_or(-1))
    }

    async fn add_post(&self, author: Uuid, th_id: i64, content: &str, post_id: Option<i64>) -> ForumResult<()> {
        sqlx::query!(
            r#" INSERT INTO forum.posts(content, author, topic, comments, created_at)
                VALUES ($1, $2, $3, $4, LOCALTIMESTAMP)"#, content, author, th_id, post_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_post(&self, p_id: i64, content: &str) -> ForumResult<()> {
        sqlx::query!(
            r#" UPDATE forum.posts
                SET content = $1
                WHERE id = $2"#, content, p_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn delete_post(&self, post_id: i64) -> ForumResult<()> {
        sqlx::query!(
            r#" DELETE FROM forum.posts
                WHERE id = $1"#, post_id)
            .execute(&self.pool)
            .await?;

        Ok(())

    }

    async fn posts_since(&self, post_id: i64) -> ForumResult<i64> {
        struct Helper {
            count: Option<i64>,
        }

        let res = sqlx::query_as!(Helper,
            r#" SELECT COUNT(*) 
                FROM forum.posts 
                WHERE created_at > (
                    SELECT created_at 
                    FROM forum.posts 
                    WHERE id = $1)"#, post_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(res.count.unwrap_or(-1))
    }
}
